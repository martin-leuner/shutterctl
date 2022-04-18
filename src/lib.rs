#[allow(dead_code, unused_imports)]
#[path = "../target/flatbuffers/shuttermsg_head_generated.rs"]
mod shutterheader;
#[allow(dead_code, unused_imports)]
#[path = "../target/flatbuffers/shuttermsg_generated.rs"]
mod shuttermsg;

mod shutterproto {
    mod transport {
        use std::io::{self, BufReader, BufWriter, Write};
        use std::net::TcpStream;

        use crate::shutterheader::{Shutterheader,
                                   ShutterheaderArgs,
                                   CryptoParam,
                                   NaClSecretBox,
                                   NaClSecretBoxArgs,
                                   Plain,
                                   PlainArgs,
                                   Version};

        pub struct Session {
            reader: BufReader<TcpStream>,
            writer: BufWriter<TcpStream>,
            id: u8,
        }

        impl Session {
            pub fn new(stream: TcpStream) -> io::Result<Self> {
                let reader = io::BufReader::new(stream.try_clone()?);
                let writer = io::BufWriter::new(stream);
                Ok(Self{reader, writer, id: 0})
            }

            pub fn _auth(&mut self, _user: &str, _key: &str) -> io::Result<()> {
                // TODO
                Ok(())
            }

            fn send(&mut self, payload: &[u8]) -> io::Result<()> {
                let mut fbb = flatbuffers::FlatBufferBuilder::new();

                let (param, crypt_type) = if self.id == 0 {
                    let param = Plain::create(&mut fbb, &PlainArgs{});
                    (param.as_union_value(), CryptoParam::Plain)
                } else {
                    // TODO: wrap, fill NaClSecretBox parameters...
                    let param = NaClSecretBox::create(&mut fbb,
                                                      &NaClSecretBoxArgs{
                                                          session: self.id,
                                                          nonce: None,
                                                          mac: None
                                                      });
                    (param.as_union_value(), CryptoParam::NaClSecretBox)
                };

                let payload = fbb.create_vector_direct(payload);

                let header = Shutterheader::create(&mut fbb,
                                                   &ShutterheaderArgs{
                                                       version: Version::Initial,
                                                       crypt_type: crypt_type,
                                                       crypt: Some(param),
                                                       payload: Some(payload)
                                                   });
                fbb.finish_size_prefixed(header, Some("SHTRMSG"));

                self.writer.write_all(fbb.finished_data())?;
                Ok(())
            }

            fn _receive(&mut self) -> io::Result<&[u8]> {
                Ok(&[])
            }
        }
    }

    pub mod rpc {
        use crate::shuttermsg::{CmdSystemState,
                                CmdSystemStateArgs,
                                Message,
                                Shuttermsg,
                                ShuttermsgArgs};

        fn command_message<'a, T>(builder: &'a mut flatbuffers::FlatBufferBuilder,
                                      msg_type: Message,
                                      msg_data: flatbuffers::WIPOffset<T>) -> &'a [u8] {
            let msg = Shuttermsg::create(builder,
                                         &ShuttermsgArgs{msg_type: msg_type,
                                         msg: Some(msg_data.as_union_value())});
            builder.finish(msg, None);
            builder.finished_data()
        }

        pub fn get_state() {
            let mut fbb = flatbuffers::FlatBufferBuilder::new();
            let data = CmdSystemState::create(&mut fbb, &CmdSystemStateArgs{});
            let cmd_buf = command_message(&mut fbb, Message::CmdSystemState, data);
            // TODO: send the command buffer through a TCP stream, wait for an answer
        }
    }
}
