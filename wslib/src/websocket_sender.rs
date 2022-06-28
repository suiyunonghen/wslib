use crate::{log_msg, MsgType, WebSocketMsg};
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
                log_msg(&format!("发送消息错误：{}", e), LLError);
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
            log_msg(&format!("发送消息错误：{}",e),LLError);
            false
        },
    }
}

#[no_mangle]
pub extern "stdcall" fn send_data_copy(mut msg: WebSocketMsg, raw: usize) ->bool{
    if msg.data.len != 0 && msg.data.data != 0{
        let mut v:Vec<u8> = Vec::with_capacity(msg.data.len);
        unsafe {
            v.set_len(msg.data.len);
        }
        let raw = unsafe {
            Vec::from_raw_parts(msg.data.data as *mut u8,msg.data.len,msg.data.len)
        };
        v.copy_from_slice(raw.as_slice());
        let vbox = Box::new(v);
        msg.data.data = Box::into_raw(vbox) as usize;
        msg.data.len = -1 as isize as usize;
    }
    send_data(msg,raw)
}


#[no_mangle]
pub extern "stdcall" fn free_writer(raw_writer: usize){
    let _ = unsafe{
        Box::from_raw(raw_writer as *mut tokio::sync::mpsc::UnboundedSender<WebSocketMsg>)
    };
}