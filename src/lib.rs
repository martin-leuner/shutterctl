#[allow(dead_code, unused_imports)]
#[path = "../target/flatbuffers/shuttermsg_head_generated.rs"]
mod shutterheader;
#[allow(dead_code, unused_imports)]
#[path = "../target/flatbuffers/shuttermsg_generated.rs"]
mod shuttermsg;

mod shutterproto {
    pub enum Error {
        Io(std::io::Error),
        FB(flatbuffers::InvalidFlatbuffer),
    }

    impl From<std::io::Error> for Error {
        fn from(e: std::io::Error) -> Self {
            Error::Io(e)
        }
    }

    impl From<flatbuffers::InvalidFlatbuffer> for Error {
        fn from(e: flatbuffers::InvalidFlatbuffer) -> Self {
            Error::FB(e)
        }
    }

    type Result<T> = std::result::Result<T, Error>;


    mod transport {
        use std::convert::TryInto;
        use std::io::{self, BufRead, BufReader, BufWriter, Write};
        use std::net::TcpStream;

        use crate::shutterheader::{self,
                                   Shutterheader,
                                   ShutterheaderArgs,
                                   CryptoParam,
                                   NaClSecretBox,
                                   NaClSecretBoxArgs,
                                   Plain,
                                   PlainArgs,
                                   Version};

        const MAGIC: &[u8] = b"SHTR";

        pub struct Session<'a> {
            reader: BufReader<TcpStream>,
            writer: BufWriter<TcpStream>,
            fbb: flatbuffers::FlatBufferBuilder<'a>,
            id: u8,
        }

        impl<'a> Session<'a> {
            pub fn new(stream: TcpStream) -> crate::shutterproto::Result<Self> {
                let reader = io::BufReader::new(stream.try_clone()?);
                let writer = io::BufWriter::new(stream);
                let fbb = flatbuffers::FlatBufferBuilder::new();
                Ok(Self{reader, writer, fbb, id: 0})
            }

            pub fn _auth(&mut self, _user: &str, _key: &str) -> crate::shutterproto::Result<()> {
                // TODO
                Ok(())
            }

            pub fn send(&mut self, payload: &[u8]) -> crate::shutterproto::Result<()> {
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
                self.fbb.finish(header, None);
                let buf = self.fbb.finished_data();
                let buf_len: u32 = buf.len().try_into().unwrap();

                self.writer.write_all(MAGIC)?;
                self.writer.write_all(&buf_len.to_le_bytes())?;
                self.writer.write_all(self.fbb.finished_data())?;
                self.writer.flush()?;
                Ok(())
            }

            pub fn receive(&mut self) -> crate::shutterproto::Result<&[u8]> {
                let answ = self.reader.fill_buf()?;
                // TODO: read magic & size
                let header = shutterheader::root_as_shutterheader(answ)?;
                // TODO: unwrap payload
                Ok(&[])
            }

            #[inline]
            pub fn exec_cmd(&mut self, payload: &[u8]) -> crate::shutterproto::Result<&[u8]> {
                self.send(payload)?;
                self.receive()
            }
        }
    }

    pub mod rpc {
        use std::net::TcpStream;

        use crate::shuttermsg::{CmdSystemState,
                                CmdSystemStateArgs,
                                Message,
                                Shuttermsg,
                                ShuttermsgArgs};
        use crate::shutterproto::transport::Session;


        fn command_message<'b, T>(fbb: &'b mut flatbuffers::FlatBufferBuilder,
                                  msg_type: Message,
                                  msg_data: flatbuffers::WIPOffset<T>) -> &'b [u8] {
            let msg = Shuttermsg::create(fbb,
                                         &ShuttermsgArgs{msg_type: msg_type,
                                         msg: Some(msg_data.as_union_value())});
            fbb.finish(msg, None);
            fbb.finished_data()
        }

        pub struct Conn<'a> {
            session: Session<'a>,
            fbb: flatbuffers::FlatBufferBuilder<'a>,
        }

        impl<'a> Conn<'a> {
            pub fn new(stream: TcpStream) -> crate::shutterproto::Result<Self> {
                let session = Session::new(stream)?;
                let fbb = flatbuffers::FlatBufferBuilder::new();
                Ok(Self{session, fbb})
            }

            pub fn get_state(&mut self) {
                self.fbb.reset();
                let data = CmdSystemState::create(&mut self.fbb, &CmdSystemStateArgs{});
                let cmd_buf = command_message(&mut self.fbb, Message::CmdSystemState, data);
                let _answ = self.session.exec_cmd(cmd_buf);

                // TODO: parse answ as Shuttermsg flatbuffer containing a RspSystemState
            }
        }
    }
}
