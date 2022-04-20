#[allow(dead_code, unused_imports)]
#[path = "../target/flatbuffers/shuttermsg_head_generated.rs"]
mod shutterheader;
#[allow(dead_code, unused_imports)]
#[path = "../target/flatbuffers/shuttermsg_generated.rs"]
mod shuttermsg;

mod shutterproto {
    use std::fmt;

    #[derive(Debug)]
    pub enum Error {
        Io(std::io::Error),
        FB(flatbuffers::InvalidFlatbuffer),
        HeaderSize,
        BadMagic,
        PayloadSize,
        UnknownVersion,
        UnknownCrypto,
        CommandMissing,
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Error::Io(e) => {
                    write!(f, "{}", e)
                }
                Error::FB(e) => {
                    write!(f, "{}", e)
                }
                Error::HeaderSize => {
                    write!(f, "Message too short to contain protocol header")
                }
                Error::BadMagic => {
                    write!(f, "Message does not start with magic number")
                }
                Error::PayloadSize => {
                    write!(f, "Message too short to contain full payload")
                }
                Error::UnknownVersion => {
                    write!(f, "Unknown protocol version")
                }
                Error::UnknownCrypto => {
                    write!(f, "Unknown crypto mechanism")
                }
                Error::CommandMissing => {
                    write!(f, "No command message enveloped in header")
                }
            }
        }
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
        use crate::shutterproto::{Error, Result};


        const MAGIC: &[u8] = b"SHTR";

        pub struct Session<'a> {
            reader: BufReader<TcpStream>,
            writer: BufWriter<TcpStream>,
            fbb: flatbuffers::FlatBufferBuilder<'a>,
            id: u8,
        }

        impl<'a> Session<'a> {
            pub fn new(stream: TcpStream) -> Result<Self> {
                let reader = io::BufReader::new(stream.try_clone()?);
                let writer = io::BufWriter::new(stream);
                let fbb = flatbuffers::FlatBufferBuilder::new();
                Ok(Self{reader, writer, fbb, id: 0})
            }

            pub fn _auth(&mut self, _user: &str, _key: &str) -> Result<()> {
                // TODO
                Ok(())
            }

            #[inline]
            pub fn send(&mut self, payload: &[u8]) -> Result<()> {
                self.build_shutterheader_fb(payload);
                self.tcp_write()
            }

            #[inline]
            pub fn receive(&mut self) -> Result<Vec<u8>> {
                let fb = self.tcp_read()?;
                Ok(self.parse_shutterheader_fb(&fb)?.to_vec())
            }

            #[inline]
            pub fn exec_cmd(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
                self.send(payload)?;
                self.receive()
            }

            fn build_shutterheader_fb(&mut self, payload: &[u8]) {
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
            }

            fn parse_shutterheader_fb<'b>(&mut self, fb: &'b [u8]) -> Result<&'b [u8]> {
                let header = shutterheader::root_as_shutterheader(fb)?;
                if header.version() != Version::Initial {
                    return Err(Error::UnknownVersion);
                }
                let payload = header.payload();
                if payload.is_none() {
                    return Err(Error::CommandMissing);
                }
                let payload = payload.unwrap();
                match header.crypt_type() {
                    CryptoParam::Plain => {
                        // Nothing to do
                    }
                    CryptoParam::NaClSecretBox => {
                        // TODO
                    }
                    _ => {
                        return Err(Error::UnknownCrypto);
                    }
                }
                Ok(payload)
            }

            fn tcp_write(&mut self) -> Result<()> {
                let buf = self.fbb.finished_data();
                let buf_len: u32 = buf.len().try_into().unwrap();

                self.writer.write_all(MAGIC)?;
                self.writer.write_all(&buf_len.to_le_bytes())?;
                self.writer.write_all(buf)?;
                self.writer.flush()?;
                Ok(())
            }

            fn tcp_read(&mut self) -> Result<Vec<u8>> {
                let mut answ = self.reader.fill_buf()?.to_vec();
                let prefix_size = MAGIC.len() + std::mem::size_of::<u32>();
                if answ.len() < prefix_size {
                    return Err(Error::HeaderSize);
                } else if &answ[..MAGIC.len()] != MAGIC {
                    return Err(Error::BadMagic);
                }
                let expected_size: usize =
                    u32::from_le_bytes(answ[MAGIC.len()..prefix_size]
                                       .try_into().unwrap())
                    .try_into().unwrap();
                let total_size = prefix_size + expected_size;
                if answ.len() < total_size {
                    return Err(Error::PayloadSize)
                }
                self.reader.consume(total_size);
                answ.drain(total_size..);
                answ.drain(..prefix_size);
                Ok(answ)
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
        use crate::shutterproto::{Result, transport::Session};


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
            pub fn new(stream: TcpStream) -> Result<Self> {
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
