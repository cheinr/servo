/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The context within which style is calculated.

use animation::Animation;
use app_units::Au;
use dom::OpaqueNode;
use error_reporting::ParseErrorReporter;
use euclid::Size2D;
use matching::StyleSharingCandidateCache;
use parking_lot::RwLock;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use stylist::Stylist;
use timer::Timer;

/// This structure is used to create a local style context from a shared one.
pub struct ThreadLocalStyleContextCreationInfo {
    new_animations_sender: Sender<Animation>,
}

impl ThreadLocalStyleContextCreationInfo {
    pub fn new(animations_sender: Sender<Animation>) -> Self {
        ThreadLocalStyleContextCreationInfo {
            new_animations_sender: animations_sender,
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum QuirksMode {
    Quirks,
    LimitedQuirks,
    NoQuirks,
}

pub struct SharedStyleContext {
    /// The current viewport size.
    pub viewport_size: Size2D<Au>,

    /// Screen sized changed?
    pub screen_size_changed: bool,

    /// The CSS selector stylist.
    pub stylist: Arc<Stylist>,

    /// Starts at zero, and increased by one every time a layout completes.
    /// This can be used to easily check for invalid stale data.
    pub generation: u32,

    /// Why is this reflow occurring
    pub goal: ReflowGoal,

    /// The animations that are currently running.
    pub running_animations: Arc<RwLock<HashMap<OpaqueNode, Vec<Animation>>>>,

    /// The list of animations that have expired since the last style recalculation.
    pub expired_animations: Arc<RwLock<HashMap<OpaqueNode, Vec<Animation>>>>,

    ///The CSS error reporter for all CSS loaded in this layout thread
    pub error_reporter: Box<ParseErrorReporter + Sync>,

    /// Data needed to create the thread-local style context from the shared one.
    pub local_context_creation_data: Mutex<ThreadLocalStyleContextCreationInfo>,

    /// The current timer for transitions and animations. This is needed to test
    /// them.
    pub timer: Timer,

    /// The QuirksMode state which the document needs to be rendered with
    pub quirks_mode: QuirksMode,
}

pub struct ThreadLocalStyleContext {
    pub style_sharing_candidate_cache: RefCell<StyleSharingCandidateCache>,
    /// A channel on which new animations that have been triggered by style
    /// recalculation can be sent.
    pub new_animations_sender: Sender<Animation>,
}

impl ThreadLocalStyleContext {
    pub fn new(local_context_creation_data: &ThreadLocalStyleContextCreationInfo) -> Self {
        ThreadLocalStyleContext {
            style_sharing_candidate_cache: RefCell::new(StyleSharingCandidateCache::new()),
            new_animations_sender: local_context_creation_data.new_animations_sender.clone(),
        }
    }
}

pub struct StyleContext<'a> {
    pub shared: &'a SharedStyleContext,
    pub thread_local: &'a ThreadLocalStyleContext,
}

/// Why we're doing reflow.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ReflowGoal {
    /// We're reflowing in order to send a display list to the screen.
    ForDisplay,
    /// We're reflowing in order to satisfy a script query. No display list will be created.
    ForScriptQuery,
}
