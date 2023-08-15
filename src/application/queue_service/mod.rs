pub mod ventrix_queue;

use crate::domain::models::service::Service;
use std::collections::{HashMap, HashSet};

type UniqueServiceList = HashSet<Service>;
type EventServiceMap = HashMap<String, UniqueServiceList>;

pub enum ListenToEventResult {
    NewEntry,
    Existed,
}
