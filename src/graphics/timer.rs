use std::collections::{hash_map::Entry, HashMap};

use windows::{core::*, Win32::Graphics::Direct3D12::*};

use super::{command_list::CommandList, device::Device, resource::Resource};

const QUERY_SIZE: u32 = 8;
const MAX_QUERY: u32 = 64;

pub struct Timer {
    heap: ID3D12QueryHeap,
    buffer: Resource,
    index: u32,
    stamps: HashMap<String, (Option<u32>, Option<u32>)>,
}

impl Timer {
    pub fn new(device: &Device) -> Result<Self> {
        let heap = device.create_query_heap(D3D12_QUERY_HEAP_TYPE_TIMESTAMP, MAX_QUERY)?;
        let buffer = Resource::new_staging_buffer(device, (QUERY_SIZE * MAX_QUERY) as _)?;

        Ok(Self {
            heap,
            buffer,
            index: 0,
            stamps: HashMap::new(),
        })
    }

    pub fn start(&mut self, command_list: &CommandList, tag: &str) {
        if self.index < MAX_QUERY {
            command_list.end_query(&self.heap, D3D12_QUERY_TYPE_TIMESTAMP, self.index);
            self.add_timestamp(tag);
        }
    }

    pub fn stop(&mut self, command_list: &CommandList, tag: &str) {
        if self.index < MAX_QUERY {
            command_list.end_query(&self.heap, D3D12_QUERY_TYPE_TIMESTAMP, self.index);
            self.add_timestamp(tag);
        }
    }

    pub fn resolve(&mut self, command_list: &CommandList) {
        if cfg!(debug_assertions) {
            command_list.resolve_query_data(
                &self.heap,
                D3D12_QUERY_TYPE_TIMESTAMP,
                self.index,
                &self.buffer.resource,
            );
        }
    }

    pub fn dump(&mut self, freq: u64) -> Result<()> {
        if cfg!(debug_assertions) {
            let data: Vec<u64> = self.buffer.read(self.index as usize)?;

            for (tag, (start, stop)) in self.stamps.iter() {
                match (start, stop) {
                    (Some(start), Some(stop)) => {
                        println!(
                            "{} {}ms",
                            tag,
                            1000.0 * (data[*stop as usize] - data[*start as usize]) as f64
                                / freq as f64
                        );
                    }
                    _ => {}
                }
            }

            self.index = 0;
            self.stamps.clear();
        }
        Ok(())
    }

    fn add_timestamp(&mut self, tag: &str) {
        let index = self.index;
        self.index += 1;

        match self.stamps.entry(tag.to_owned()) {
            Entry::Occupied(mut e) => {
                e.get_mut().1 = Some(index);
            }
            Entry::Vacant(e) => {
                e.insert((Some(index), None));
            }
        }
    }
}
