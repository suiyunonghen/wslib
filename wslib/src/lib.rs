extern crate core;
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
//use std::future::Future;
use std::net::Ipv6Addr;
use std::sync::Arc;
//use std::sync::atomic::Ordering;
//use std::vec;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use crate::tungstenite::Message;
use crate::tungstenite::protocol::CloseFrame;

pub mod websocket_srv;
pub mod client;
mod websocket_sender;


#[derive(Copy,Clone)]
#[repr(C)]
pub struct StringData{
    data: usize, //*mut u8,
    len: usize,
}

impl StringData {
    pub fn empty()->Self{
        Self{
            data: 0,// as *mut u8,
            len: 0,
        }
    }
}

#[derive(Copy,Clone)]
#[repr(C)]
pub struct WebSocketMsg{
    msg_type:   u16,
    code:       u16,
    data:       StringData,
}

impl WebSocketMsg {
    pub fn message(&self)->Option<Message>{
        match MsgType::from(self.msg_type) {
            MsgType::Text=>{
                if self.data.data == 0 || self.data.len == 0{
                    return Some(Message::Text("".to_string()));
                }
                if self.data.len as isize != -1{
                    let s = unsafe{
                        String::from_raw_parts(self.data.data as *mut u8,self.data.len,self.data.len)
                    };
                    std::mem::forget(s);
                    let s = unsafe{
                        String::from_raw_parts(self.data.data as *mut u8,self.data.len,self.data.len)
                    };
                    Some(Message::Text(s))
                }else{
                    let v = unsafe{
                        Box::from_raw(self.data.data as *mut Vec<u8>)
                    };
                    if let Ok(s) = String::from_utf8(*v){
                        Some(Message::Text(s))
                    }else{
                        None
                    }
                }
            },
            MsgType::Binary=>{
                if self.data.data == 0 || self.data.len == 0{
                    return Some(Message::Binary(Vec::new()));
                }
                if self.data.len as isize == -1{
                    let v = unsafe{
                        Box::from_raw(self.data.data as *mut Vec<u8>)
                    };
                    return Some(Message::Binary(*v));
                }
                let s = unsafe{
                    Vec::from_raw_parts(self.data.data as *mut u8,self.data.len,self.data.len)
                };
                std::mem::forget(s);
                let s = unsafe{
                    Vec::from_raw_parts(self.data.data as *mut u8,self.data.len,self.data.len)
                };
                Some(Message::Binary(s))
            },
            MsgType::Ping=>{
                if self.data.data == 0 || self.data.len == 0{
                    return Some(Message::Ping(Vec::new()));
                }
                if self.data.len as isize == -1{
                    let v = unsafe{
                        Box::from_raw(self.data.data as *mut Vec<u8>)
                    };
                    return Some(Message::Ping(*v));
                }
                let s = unsafe{
                    Vec::from_raw_parts(self.data.data as *mut u8,self.data.len,self.data.len)
                };
                std::mem::forget(s);
                let s = unsafe{
                    Vec::from_raw_parts(self.data.data as *mut u8,self.data.len,self.data.len)
                };
                Some(Message::Ping(s))
            },
            MsgType::Pong=>{
                if self.data.data == 0 || self.data.len == 0{
                    return Some(Message::Pong(Vec::new()));
                }
                if self.data.len as isize == -1{
                    let v = unsafe{
                        Box::from_raw(self.data.data as *mut Vec<u8>)
                    };
                    return Some(Message::Pong(*v));
                }
                let s = unsafe{
                    Vec::from_raw_parts(self.data.data as *mut u8,self.data.len,self.data.len)
                };
                std::mem::forget(s);
                let s = unsafe{
                    Vec::from_raw_parts(self.data.data as *mut u8,self.data.len,self.data.len)
                };
                Some(Message::Pong(s))
            },
            MsgType::Close=>{
                let mut frame = CloseFrame{
                    code: CloseCode::from(self.code),
                    reason: Default::default(),
                };
                if self.data.data != 0 && self.data.len != 0{
                    if self.data.len as isize == -1{
                        let v = unsafe{
                            Box::from_raw(self.data.data as *mut Vec<u8>)
                        };
                        return match String::from_utf8(*v){
                          Ok(s)=>Some(Message::Text(s)),
                          _=>None,
                        };
                    }
                    let s = unsafe{
                        String::from_raw_parts(self.data.data as *mut u8,self.data.len,self.data.len)
                    };
                    std::mem::forget(s);
                    let s = unsafe{
                        String::from_raw_parts(self.data.data as *mut u8,self.data.len,self.data.len)
                    };
                    frame.reason = Cow::Owned(s);
                }
                Some(Message::Close(Some(frame)))
            },
            _=>None,
        }
    }

    pub fn default()->Self{
        WebSocketMsg{
            msg_type: 6,
            code: 0,
            data: StringData::empty(),
        }
    }
}

pub type LogCallBack = extern "stdcall" fn(log_level: isize,log_info: StringData);
pub type WebSocketRecvCallBack = extern "stdcall" fn(msg_type: i8,data: StringData,client: usize,manager: usize);
pub type WebSocketSendCallBack = extern "stdcall" fn(succeed: bool, msg: WebSocketMsg, client: usize, manager: usize);
pub type WebSocketCloseCallBack = extern "stdcall" fn(code: u16,reason: StringData,client: usize, manager: usize);
pub type WebSocketConnectedCallBack = extern "stdcall" fn(connected: bool,socket_write: usize,socket_addr: *const SocketAddrInfo, manager: usize)->usize; //返回client
pub enum LogLevel {
    LLDebug = 0,
    LLInfo = 1,
    LLWarn,
    LLError,
    LLException,
    LLPanic,
}

impl LogLevel {
    pub fn value(&self) -> isize {
        match *self {
            LogLevel::LLDebug => 0,
            LogLevel::LLInfo => 1,
            LogLevel::LLWarn => 2,
            LogLevel::LLError => 3,
            LogLevel::LLException => 4,
            LogLevel::LLPanic => 5,
        }
    }
}

#[derive(Copy, Clone)]
pub enum MsgType{
    Text,
    Binary,
    Ping,
    Pong,
    Close,
    Frame,

    None,
}

impl From<MsgType> for u16 {
    fn from(code: MsgType) -> u16 {
        match code {
            MsgType::Text=>0,
            MsgType::Binary=>1,
            MsgType::Ping=>2,
            MsgType::Pong=>3,
            MsgType::Close=>4,
            MsgType::Frame=>5,
            _=>6,
        }
    }
}

impl From<u16> for MsgType {
    fn from(code: u16) -> MsgType {
        match code {
            0=>MsgType::Text,
            1=>MsgType::Binary,
            2=>MsgType::Ping,
            3=>MsgType::Pong,
            4=>MsgType::Close,
            5=>MsgType::Frame,
            _=>MsgType::None,
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone)]
pub struct SocketAddrInfo{
    ip_v6: bool,        //默认为ipv4
    port:  u16,         //端口
    ip:    [u8;16],     //ip地址
}

impl Display for SocketAddrInfo {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        if !self.ip_v6{
            write!(fmt, "{}.{}.{}.{}:{}", self.ip[0], self.ip[1], self.ip[2], self.ip[3],self.port)
        }else{
            write!(fmt,"{}:{}",Ipv6Addr::from(self.ip).to_string(),self.port)
        }
    }
}

pub static mut LOG: Option<LogCallBack> = None;
pub static mut RUNTIME_NOTIFY: Option<Arc<tokio::sync::watch::Sender<Option<()>>>> = None;

pub fn log_msg(msg: &str, level: LogLevel){
    unsafe {
        if let Some(ref callback) = LOG{
            (*callback)(level.value(),StringData{
                data: msg.as_ptr() as usize,
                len: msg.len() as usize,
            });
        }
    }
}


pub fn process_msg(client: usize,manager: usize, message: tungstenite::Message,recv: Option<WebSocketRecvCallBack>)->MsgType{
    if let Some(recv) = recv{
        let mut data = StringData::empty();
        let result: MsgType;
        match message {
            tungstenite::Message::Text(ref txt)=>{
                data.len = txt.len();
                if data.len > 0{
                    data.data = txt.as_bytes().as_ptr() as usize;
                }
                result = MsgType::Text;
                recv(result as i8,data,client,manager);
            },
            tungstenite::Message::Binary(ref bin)=>{
                data.len = bin.len();
                if data.len > 0{
                    data.data = bin.as_slice().as_ptr() as usize;
                }
                result = MsgType::Binary;
                recv(result as i8,data,client,manager);
            },
            tungstenite::Message::Close(close)=>{
                if let Some(frame) = close{
                    data.len = frame.reason.len();
                    if data.len > 0{
                        data.data = frame.reason.as_ptr() as usize;
                    }
                };
                result = MsgType::Close;
                recv(result as i8,data,client,manager);
            },
            tungstenite::Message::Ping(ref ping_data)=>{
                data.len = ping_data.len();
                if data.len > 0{
                    data.data = ping_data.as_slice().as_ptr() as usize;
                }
                result = MsgType::Ping;
                recv(result as i8,data,client,manager);
            },
            tungstenite::Message::Pong(pong)=>{
                data.len = pong.len();
                if data.len > 0{
                    data.data = pong.as_slice().as_ptr() as usize;
                }
                result = MsgType::Pong;
                recv(result as i8,data,client,manager);
            },
            tungstenite::Message::Frame(_)=>{
                result = MsgType::Frame;
            },
        }
        return result;
    }
    MsgType::None
}

#[no_mangle]
pub extern "stdcall" fn init_ws_runtime(max_threads: u16,block_func: Option<extern "stdcall" fn()>,log: Option<LogCallBack>){
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    if max_threads > 0{
        builder.worker_threads(max_threads as usize);
    }
    unsafe {
        LOG = log;
    }
    builder.thread_keep_alive(tokio::time::Duration::from_secs(5)).enable_io().enable_time();
    if let Ok(runtime) = builder.build(){
        if let Some(func) = block_func{
            runtime.block_on(async move{
                func();
            });
        }else{
            let (tx,mut rx) = tokio::sync::watch::channel(Option::<()>::None);
            std::thread::spawn(move||{
                async move{
                    let _ = rx.changed().await;
                }
            });
            let tx = Arc::new(tx);
            unsafe {
                RUNTIME_NOTIFY = Some(tx);
            }
            /*let tx = Box::new(tx);
            return Box::into_raw(tx) as *const Arc<tokio::sync::watch::Sender<Option<()>>> as usize;*/
        }
    }
}

#[no_mangle]
pub extern "stdcall" fn finalize_ws_runtime(){
    unsafe {
        if let Some(ref tx) = RUNTIME_NOTIFY{
            let _ = tx.send(Some(()));
            RUNTIME_NOTIFY = None;
        }
    }
}


#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tokio::io::{AsyncWriteExt};
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::{self,handshake::server::{Response as HandleShakeResponse,ErrorResponse},http::{Request as HandleShakeRequest,StatusCode}};


    async fn test_websocket(){
        //等待100ms
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let mut stdout = tokio::io::stdout();
        stdout.write("准备发送Websocket了\r\n".as_bytes()).await.expect("TODO: panic message");
        match tokio_tungstenite::connect_async("ws://127.0.0.1:8088/test").await{
            Ok((stream,_))=>{
                let _ = tokio::io::stdout().write("打开连接成功\r\n".as_bytes()).await;
                let (mut ws_sender, mut ws_receiver) = stream.split();
                ws_sender.send(tokio_tungstenite::tungstenite::Message::Text("发送第一波消息了".to_string())).await.expect("TODO: panic message");
                let mut interval = tokio::time::interval(Duration::from_millis(5000));
                let mut idx = 0;
                loop {
                    tokio::select! {
                        msg=ws_receiver.next()=>{
                            if let Some(msg)=msg{
                                if let Ok(message) = msg{
                                    match message {
                                        tokio_tungstenite::tungstenite::Message::Text(ref txt)=>{
                                            let _ = stdout.write(format!("服务回消息了额：{}\r\n",txt).as_bytes()).await;
                                        },
                                        tokio_tungstenite::tungstenite::Message::Binary(_)=>{

                                        },
                                        tokio_tungstenite::tungstenite::Message::Ping(data)=>{
                                            //回复pong
                                            ws_sender.send(tokio_tungstenite::tungstenite::Message::Pong(data)).await.unwrap();
                                        },
                                        tokio_tungstenite::tungstenite::Message::Close(_)=>{
                                            let _ = stdout.write("正常关闭\r\n".as_bytes()).await;
                                            break;
                                        },
                                    _=>(),
                                    }
                                }
                            }else{
                                break;
                            }
                        },
                        _=interval.tick()=>{
                            idx = idx + 1;
                            if idx >= 60{
                                //关闭了
                                break;
                            }
                            let _ = ws_sender.send(tungstenite::Message::Text("gsdfgsdfgsdfgsgsdfgsdgf".to_string())).await;
                        }
                    }
                }
            },
            Err(e)=>{
                let _ = tokio::io::stdout().write(format!("发生错误了：{:?}",e).as_bytes()).await;
            }
        }
    }


    fn handle_shake(req: &HandleShakeRequest<()>, mut res: HandleShakeResponse)->Result<HandleShakeResponse, ErrorResponse>{
        if req.uri().path().eq_ignore_ascii_case("/ttg"){

        }else{
            *res.status_mut() = StatusCode::BAD_REQUEST;
        }
        Ok(res)
    }

    async fn handle_connection(stream: tokio::net::TcpStream){
        let ws_stream = tokio_tungstenite::accept_hdr_async(stream,handle_shake).await.expect("accept error");
        let (mut ws_sender,mut ws_receiver) = ws_stream.split();
        let mut interval = tokio::time::interval(Duration::from_millis(500));
        let mut stdout = tokio::io::stdout();
        loop{
            tokio::select! {
                msg = ws_receiver.next()=>{
                    if let Some(msg) = msg{
                        match msg  {
                            Ok(message) =>{
                                match message {
                                    tungstenite::Message::Text(ref txt)=>{
                                        let _ = stdout.write(format!("接收到消息了额：{}\r\n",txt).as_bytes()).await;
                                        ws_sender.send(message).await.unwrap();
                                        //ws_sender.send(tungstenite::Message::Close(None)).await.unwrap();
                                        let _ = ws_sender.close().await;
                                    },
                                    tungstenite::Message::Binary(_)=>{
                                        ws_sender.send(message).await.unwrap();
                                    },
                                    tungstenite::Message::Ping(data)=>{
                                        //回复pong
                                        ws_sender.send(tungstenite::Message::Pong(data)).await.unwrap();
                                    },
                                    tungstenite::Message::Close(_)=>{
                                        let _ = stdout.write("正常关闭\r\n".as_bytes()).await;
                                        break;
                                    },
                                    _=>(),
                                }
                            },
                            Err(e)=>{
                                //发生错误了
                                let _ = tokio::io::stderr().write(format!("发生错误了啦：{}\r\n",e).as_bytes()).await;
                                break;
                            }
                        }
                    }else{
                        //完毕了，
                        break;
                    }
                },
                _=interval.tick()=>{
                    let _ = ws_sender.send(tungstenite::Message::Text("定时发送消息".to_owned())).await;
                }
            }
        }
    }

    async fn test_websocket_srv(){
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8088").await.expect("listen错误");
        let sleep = tokio::time::sleep(tokio::time::Duration::from_secs(5));
        let accept = listener.accept();
        tokio::pin!(sleep);
        let mut stdout = tokio::io::stdout();
        tokio::select! {
            _ = &mut sleep =>{
                let _ = stdout.write("准备结束了\r\n".as_bytes()).await;
            },
            //tokio_native_tls::TlsAcceptor::accept()
           result = accept =>{
                //只接收一次
                if let Ok((stream,_)) = result{
                    /*let acceptor = acceptor.clone();
                    let mut stream = acceptor.accept(stream).await.unwrap();*/
                    handle_connection(stream).await;
                }else{
                    let _ = stdout.write("发生错误了\r\n".as_bytes()).await.unwrap();
                }
            }
        }
       /* while let Ok((stream,m)) = listener.accept().await{

        }*/
    }

    #[test]
    fn it_works() {
        let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
        runtime.spawn(test_websocket());
        runtime.block_on(test_websocket_srv());
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_client(){
        let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
        runtime.block_on(test_websocket());
    }
}
