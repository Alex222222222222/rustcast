use std::collections::{BTreeSet, HashMap};

pub struct ListenerFrameData {
    /// listener id to frame id
    data: HashMap<usize, usize>,
    /// frame id and listener id
    index: BTreeSet<(usize, usize)>,
}

impl ListenerFrameData {
    /// new
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            index: BTreeSet::new(),
        }
    }

    /// delete a listener from the playlist
    pub fn delete_listener_data(&mut self, listener_id: usize) {
        let frame_id = self.data.remove(&listener_id);
        if let Some(frame_id) = frame_id {
            self.index.remove(&(frame_id, listener_id));
        }
    }

    /// log the listener current frame
    pub fn log_current_frame(&mut self, listener_id: usize, frame_id: usize) {
        self.delete_listener_data(listener_id);
        self.data.insert(listener_id, frame_id);
        self.index.insert((frame_id, listener_id));
    }

    /// get the smallest frame id in ListenerFrame
    pub async fn get_smallest_frame_id(&self) -> Option<usize> {
        self.index.first().map(|(frame_id, _)| *frame_id)
    }
}
