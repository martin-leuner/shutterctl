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


        pub struct Session<'a> {
            reader: BufReader<TcpStream>,
            writer: BufWriter<TcpStream>,
            fbb: flatbuffers::FlatBufferBuilder<'a>,
            id: u8,
        }

        impl<'a> Session<'a> {
            pub fn new(stream: TcpStream) -> io::Result<Self> {
                let reader = io::BufReader::new(stream.try_clone()?);
                let writer = io::BufWriter::new(stream);
                let fbb = flatbuffers::FlatBufferBuilder::new();
                Ok(Self{reader, writer, fbb, id: 0})
            }

            pub fn _auth(&mut self, _user: &str, _key: &str) -> io::Result<()> {
                // TODO
                Ok(())
            }

            fn send(&mut self, payload: &[u8]) -> io::Result<()> {
                self.fbb.reset();

                let (param, crypt_type) = if self.id == 0 {
                    let param = Plain::create(&mut self.fbb, &PlainArgs{});
                    (param.as_union_value(), CryptoParam::Plain)
                } else {
                    // TODO: wrap, fill NaClSecretBox parameters...
                    let param = NaClSecretBox::create(&mut self.fbb,
                                                      &NaClSecretBoxArgs{
                                                          session: self.id,
                                                          nonce: None,
                                                          mac: None
                                                      });
                    (param.as_union_value(), CryptoParam::NaClSecretBox)
                };

                let payload = self.fbb.create_vector_direct(payload);

                let header = Shutterheader::create(&mut self.fbb,
                                                   &ShutterheaderArgs{
                                                       version: Version::Initial,
                                                       crypt_type: crypt_type,
                                                       crypt: Some(param),
                                                       payload: Some(payload)
                                                   });
                self.fbb.finish_size_prefixed(header, Some("SHTRMSG"));

                self.writer.write_all(self.fbb.finished_data())?;
                Ok(())
            }

            fn _receive(&mut self) -> io::Result<&[u8]> {
                Ok(&[])
            }
        }
    }

    pub mod rpc {
        use std::io;
        use std::net::TcpStream;

        use crate::shuttermsg::{CmdSystemState,
                                CmdSystemStateArgs,
                                Message,
                                Shuttermsg,
                                ShuttermsgArgs};
        use crate::shutterproto::transport::Session;


        pub struct Conn<'a> {
            session: Session<'a>,
            fbb: flatbuffers::FlatBufferBuilder<'a>,
        }

        impl<'a> Conn<'a> {
            pub fn new(stream: TcpStream) -> io::Result<Self> {
                let session = Session::new(stream)?;
                let fbb = flatbuffers::FlatBufferBuilder::new();
                Ok(Self{session, fbb})
            }

            fn command_message<T>(&mut self,
                                  msg_type: Message,
                                  msg_data: flatbuffers::WIPOffset<T>) -> &[u8] {
                let msg = Shuttermsg::create(&mut self.fbb,
                                             &ShuttermsgArgs{msg_type: msg_type,
                                             msg: Some(msg_data.as_union_value())});
                self.fbb.finish(msg, None);
                self.fbb.finished_data()
            }

            pub fn get_state(&mut self) {
                self.fbb.reset();
                let data = CmdSystemState::create(&mut self.fbb, &CmdSystemStateArgs{});
                let cmd_buf = self.command_message(Message::CmdSystemState, data);
                // TODO: send the command buffer through a TCP stream, wait for an answer
            }
        }
    }
}
