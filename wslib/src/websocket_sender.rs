use crate::{log_msg,LOG, MsgType, WebSocketMsg,StringData};
use crate::LogLevel::{LLError};

#[no_mangle]
pub extern "stdcall" fn send_data(msg: WebSocketMsg,raw: usize)->bool{
    if let MsgType::Close = MsgType::from(msg.msg_type){
        //释放掉这个writer
        let sender = unsafe{
            Box::from_raw(raw as *mut tokio::sync::mpsc::UnboundedSender<WebSocketMsg>)
        };
        return match sender.send(msg) {
            Ok(()) => true,
            Err(e) => {
                log_msg!(LLError,"发送消息错误：{}", e);
                false
            },
        }
    }
    let sender = unsafe{
        let sender = raw as *mut tokio::sync::mpsc::UnboundedSender<WebSocketMsg>;
        &mut *sender
    };
    match sender.send(msg){
        Ok(())=>true,
        Err(e)=>{
            log_msg!(LLError,"发送消息错误：{}",e);
            false
        },
    }
}

#[no_mangle]
pub extern "stdcall" fn free_writer(raw_writer: usize){
    let _ = unsafe{
        Box::from_raw(raw_writer as *mut tokio::sync::mpsc::UnboundedSender<WebSocketMsg>)
    };
}