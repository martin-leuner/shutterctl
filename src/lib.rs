#[allow(dead_code, unused_imports)]
#[path = "../target/flatbuffers/shuttermsg_head_generated.rs"]
mod shutterheader;
#[allow(dead_code, unused_imports)]
#[path = "../target/flatbuffers/shuttermsg_generated.rs"]
mod shuttermsg;

mod shutterproto {
    pub mod rpc {
        use crate::shuttermsg::{CmdSystemState,
                                CmdSystemStateArgs,
                                Message,
                                Shuttermsg,
                                ShuttermsgArgs};

        pub fn command_message<'a, T>(builder: &'a mut flatbuffers::FlatBufferBuilder,
                                      msg_type: Message,
                                      msg_data: flatbuffers::WIPOffset<T>) -> &'a [u8]
        {
            let msg = Shuttermsg::create(builder,
                                         &ShuttermsgArgs{msg_type: msg_type,
                                         msg: Some(msg_data.as_union_value())});
            builder.finish(msg, None);
            builder.finished_data()
        }

        pub fn get_state()
        {
            let mut fbb = flatbuffers::FlatBufferBuilder::new();
            let data = CmdSystemState::create(&mut fbb, &CmdSystemStateArgs{});
            let cmd_buf = command_message(&mut fbb, Message::CmdSystemState, data);
            // TODO: send the command buffer through a TCP stream, wait for an answer
        }
    }
}
