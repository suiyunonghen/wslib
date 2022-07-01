extern crate core;
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::net::Ipv6Addr;
use std::sync::Arc;
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
    pub fn message(&self,copy_data: bool)->(Option<Message>,Option<Vec<u8>>){
        match MsgType::from(self.msg_type) {
            MsgType::Text=>{
                if self.data.data == 0 || self.data.len == 0{
                    return (Some(Message::Text("".to_string())),None);
                }
                if self.data.len as isize != -1{
                    let s = unsafe{
                        String::from_raw_parts(self.data.data as *mut u8,self.data.len,self.data.len)
                    };
                    if !copy_data{
                        (Some(Message::Text(s)),None)
                    }else{
                        let v:Vec<u8> = Vec::from(s.as_str());
                        (Some(Message::Text(s)),Some(v))
                    }
                }else{
                    let v = unsafe{
                        Box::from_raw(self.data.data as *mut Vec<u8>)
                    };
                    if let Ok(s) = String::from_utf8(*v){
                        if !copy_data{
                            (Some(Message::Text(s)),None)
                        }else{
                            let v_result = Vec::from(s.as_str());
                            (Some(Message::Text(s)),Some(v_result))
                        }
                    }else{
                        (None,None)
                    }
                }
            },
            MsgType::Binary=>{
                if self.data.data == 0 || self.data.len == 0{
                    return (Some(Message::Binary(Vec::new())),None);
                }
                if self.data.len as isize == -1{
                    let v = unsafe{
                        Box::from_raw(self.data.data as *mut Vec<u8>)
                    };
                    if !copy_data{
                        return (Some(Message::Binary(*v)),None);
                    }
                    let v_result: Vec<u8> = Vec::from(v.as_slice());
                    return (Some(Message::Binary(*v)),Some(v_result));
                }
                let s = unsafe{
                    Vec::from_raw_parts(self.data.data as *mut u8,self.data.len,self.data.len)
                };
                if !copy_data{
                    return (Some(Message::Binary(s)),None);
                }
                let v = Vec::from(s.as_slice());
                (Some(Message::Binary(s)),Some(v))
            },
            MsgType::Ping=>{
                if self.data.data == 0 || self.data.len == 0{
                    return (Some(Message::Ping(Vec::new())),None);
                }
                if self.data.len as isize == -1{
                    let v = unsafe{
                        Box::from_raw(self.data.data as *mut Vec<u8>)
                    };
                    if !copy_data{
                        return (Some(Message::Ping(*v)),None);
                    }
                    let vdata = Vec::from(v.as_slice());
                    return (Some(Message::Ping(*v)),Some(vdata))
                }
                let s = unsafe{
                    Vec::from_raw_parts(self.data.data as *mut u8,self.data.len,self.data.len)
                };
                if !copy_data{
                    return (Some(Message::Ping(s)),None)
                }
                let v_data = Vec::from(s.as_slice());
                (Some(Message::Ping(s)),Some(v_data))
            },
            MsgType::Pong=>{
                if self.data.data == 0 || self.data.len == 0{
                    return (Some(Message::Pong(Vec::new())),None);
                }
                if self.data.len as isize == -1{
                    let v = unsafe{
                        Box::from_raw(self.data.data as *mut Vec<u8>)
                    };
                    if !copy_data{
                        return (Some(Message::Pong(*v)),None);
                    }
                    let vdata = Vec::from(v.as_slice());
                    return (Some(Message::Pong(*v)),Some(vdata))
                }
                let s = unsafe{
                    Vec::from_raw_parts(self.data.data as *mut u8,self.data.len,self.data.len)
                };
                if !copy_data{
                    return (Some(Message::Pong(s)),None)
                }
                let vdata = Vec::from(s.as_slice());
                return (Some(Message::Pong(s)),Some(vdata))
            },
            MsgType::Close=>{
                let mut frame = CloseFrame{
                    code: CloseCode::from(self.code),
                    reason: Default::default(),
                };
                let mut v_result: Option<Vec<u8>> = None;
                if self.data.data != 0 && self.data.len != 0{
                    if self.data.len as isize == -1{
                        let v = unsafe{
                            Box::from_raw(self.data.data as *mut Vec<u8>)
                        };
                        return match String::from_utf8(*v){
                          Ok(s)=>{
                              if !copy_data{
                                  frame.reason = Cow::Owned(s);
                                  (Some(Message::Close(Some(frame))),None)
                              }else{
                                  (Some(Message::Close(Some(frame))),Some(Vec::from(s.as_str())))
                              }
                          },
                          _=>(None,None),
                        };
                    }
                    let s = unsafe{
                        String::from_raw_parts(self.data.data as *mut u8,self.data.len,self.data.len)
                    };
                    v_result = Some(Vec::from(s.as_str()));
                    frame.reason = Cow::Owned(s);
                }
                (Some(Message::Close(Some(frame))),v_result)
            },
            _=>(None,None),
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

impl From<LogLevel> for isize {
    fn from(l: LogLevel) -> Self {
        match l {
            LogLevel::LLDebug => 0,
            LogLevel::LLInfo => 1,
            LogLevel::LLWarn => 2,
            LogLevel::LLError => 3,
            LogLevel::LLException => 4,
            LogLevel::LLPanic => 5,
        }
    }
}

impl From<isize> for LogLevel {
    fn from(code: isize) -> Self {
        match code {
            0=>LogLevel::LLDebug,
            1=>LogLevel::LLInfo,
            2=>LogLevel::LLWarn,
            3=>LogLevel::LLError,
            4=>LogLevel::LLException,
            5=>LogLevel::LLPanic,
            _=>LogLevel::LLDebug,
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

#[macro_export]
macro_rules! log_msg {
    ($level: expr, $($msg: expr),+) => {
        unsafe{
            if let Some(ref callback) = LOG{
                let st = format!($($msg),+);
                (*callback)(isize::from($level),StringData{
                    data: st.as_bytes().as_ptr() as usize,
                    len:  st.len(),
                });
            }
        }
    };
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

#[no_mangle]
pub extern "stdcall" fn alloc(size: usize)->usize{
    let layout = std::alloc::Layout::from_size_align(size,std::mem::size_of::<usize>()).unwrap();
    unsafe{
        std::alloc::alloc(layout) as usize
    }
}

#[no_mangle]
pub extern "stdcall" fn realloc(p: usize, old_size: usize,size: usize)->usize{
    let layout = std::alloc::Layout::from_size_align(old_size,std::mem::size_of::<usize>()).unwrap();
    unsafe {
        std::alloc::realloc(p as *mut u8,layout,size) as usize
    }
}

#[no_mangle]
pub extern "stdcall" fn dealloc(p: usize,size: usize){
    let layout = std::alloc::Layout::from_size_align(size,std::mem::size_of::<usize>()).unwrap();
    unsafe {
        std::alloc::dealloc(p as *mut u8,layout)
    }
}


#[cfg(test)]
mod tests {
    use std::io::{stdout, Write};
    use std::time::Duration;
    use tokio::io::{AsyncWriteExt};
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite;
    use crate::{finalize_ws_runtime, init_ws_runtime, MsgType, SocketAddrInfo, StringData};
    use crate::websocket_srv::{ServerCallBack, TlsFileInfo, websocket_listen};


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


    extern "stdcall" fn lib_log(log_level: isize,log_info: StringData){
        let s = unsafe{
            String::from_raw_parts(log_info.data as *mut u8,log_info.len,log_info.len)
        };
        let s = std::mem::ManuallyDrop::new(s);
        stdout().write(s.as_bytes()).unwrap();
    }


    extern "stdcall" fn handle_shake(req: usize,response: usize,manager: usize)->bool{
        true
    }

    extern "stdcall" fn client_connected(connected: bool,socket_write: usize,socket_addr: *const SocketAddrInfo, manager: usize)->usize{
        if connected{
            socket_write
        }else{
            0
        }
    }

    extern "stdcall" fn recv(msg_type: i8,data: StringData,client: usize,manager: usize){
        match MsgType::from(msg_type as u16){
            MsgType::Text=>{
                let s = unsafe{
                    String::from_raw_parts(data.data as *mut u8,data.len,data.len)
                };
                let s = std::mem::ManuallyDrop::new(s);
                stdout().write(s.as_bytes()).unwrap();
            },
            _=>{

            }
        }
    }

    async fn serve(){
        let callback = ServerCallBack{
            manager: 0,
            before_handle_shake: None,
            on_handle_shake: Some(handle_shake),
            on_connected: Some(client_connected),
            on_recv: Some(recv),
            on_send: None,
            on_closed: None
        };
        let _ = websocket_listen(8088,callback,TlsFileInfo::default());

    }

    extern "stdcall" fn main_srv(){
        tokio::spawn(serve());
        tokio::spawn(test_websocket());
        std::thread::sleep(std::time::Duration::from_secs(6));
    }

    #[test]
    fn it_works() {
        init_ws_runtime(0,Some(main_srv),Some(lib_log));
        finalize_ws_runtime();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    #[test]
    fn test_client(){
        let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
        runtime.block_on(test_websocket());
    }
}
