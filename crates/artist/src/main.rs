#![no_std]
#![no_main]
#![feature(const_trait_impl)]
#![feature(int_roundings)]
#![feature(never_type)]

extern crate alloc;

use sel4cp::memory_region::{memory_region_symbol, ExternallySharedRef, ReadOnly, ReadWrite};
use sel4cp::message::{MessageInfo, NoMessageValue, StatusMessageLabel};
use sel4cp::{protection_domain, Channel, Handler};

use banscii_artist_interface_types::*;

mod artistic_secrets;
mod cryptographic_secrets;

use artistic_secrets::Masterpiece;

const ASSISTANT: Channel = Channel::new(0);

const REGION_SIZE: usize = 0x4_000;

#[protection_domain(heap_size = 0x10000)]
fn init() -> ThisHandler {
    let region_in = unsafe {
        ExternallySharedRef::<'static, [u8]>::new_read_only(
            memory_region_symbol!(region_in_start: *mut [u8], n = REGION_SIZE),
        )
    };

    let region_out = unsafe {
        ExternallySharedRef::<'static, [u8]>::new(
            memory_region_symbol!(region_out_start: *mut [u8], n = REGION_SIZE),
        )
    };

    ThisHandler {
        region_in,
        region_out,
    }
}

struct ThisHandler {
    region_in: ExternallySharedRef<'static, [u8], ReadOnly>,
    region_out: ExternallySharedRef<'static, [u8], ReadWrite>,
}

impl Handler for ThisHandler {
    type Error = !;

    fn protected(
        &mut self,
        channel: Channel,
        msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        Ok(match channel {
            ASSISTANT => match msg_info.recv::<Request>() {
                Ok(msg) => {
                    let draft_height = msg.height;
                    let draft_width = msg.width;
                    let draft = self
                        .region_in
                        .as_ptr()
                        .index(msg.draft_start..msg.draft_start + msg.draft_size)
                        .copy_to_vec();

                    let masterpiece = Masterpiece::complete(draft_height, draft_width, &draft);

                    let masterpiece_start = 0;
                    let masterpiece_size = masterpiece.pixel_data.len();
                    let masterpiece_end = masterpiece_start + masterpiece_size;

                    self.region_out
                        .as_mut_ptr()
                        .index(masterpiece_start..masterpiece_end)
                        .copy_from_slice(&masterpiece.pixel_data);

                    let signature = cryptographic_secrets::sign(&masterpiece.pixel_data);
                    let signature = signature.as_ref();

                    let signature_start = masterpiece_end;
                    let signature_size = signature.len();
                    let signature_end = signature_start + signature_size;

                    self.region_out
                        .as_mut_ptr()
                        .index(signature_start..signature_end)
                        .copy_from_slice(&signature);

                    MessageInfo::send(
                        StatusMessageLabel::Ok,
                        Response {
                            height: masterpiece.height,
                            width: masterpiece.width,
                            masterpiece_start,
                            masterpiece_size,
                            signature_start,
                            signature_size,
                        },
                    )
                }
                Err(_) => MessageInfo::send(StatusMessageLabel::Error, NoMessageValue),
            },
            _ => {
                unreachable!()
            }
        })
    }
}
