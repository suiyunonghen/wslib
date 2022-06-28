# wslib
delphi websocket library wrapped by rust

# 说明
包装提供了两套函数，一套是send，一套是sendcopy，其中send中传递的内存地址，需要自己持有其生命周期，直到AfterSendMsg的时候，才能释放掉，通过这个机制可以使用内存池来做处理，
使用sendcopy函数的时候，其中的内存不必管，因为此函数会先将数据复制，所以可以在sendcopy之后立即释放，同时AfterSendMsg中的内存信息也是nil。

InitWslib函数中，appMainBlock主要是用来启动rust的tokio的异步阻塞运行时函数，使用Application.Run来阻塞，设置为false的时候，则不会，所以，如果要设置为true，请在应用程序开头
去掉Application.Run，然后使用InitWslib来代替
