cmd_Release/obj.target/ts.node := g++ -shared -pthread -rdynamic -m64  -Wl,-soname=ts.node -o Release/obj.target/ts.node -Wl,--start-group Release/obj.target/ts/ts.o -Wl,--end-group 
