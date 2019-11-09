var N=null,E="",T="t",U="u",searchIndex={};
var R=["usize","fnonce","option","producer","capacity","Returns capacity of the ring buffer.","is_empty","Checks if the ring buffer is empty.","is_full","Checks if the ring buffer is full.","remaining","The remaining space in the buffer.","Removes at most `count` elements from the consumer and…","consumer","result","try_from","try_into","borrow_mut","type_id","Consumer","Producer","RingBuffer"];

searchIndex["ringbuf"]={"doc":"Lock-free single-producer single-consumer (SPSC) FIFO ring…","i":[[3,R[19],"ringbuf","Consumer part of ring buffer.",N,N],[3,R[20],E,"Producer part of ring buffer.",N,N],[3,R[21],E,"Ring buffer itself.",N,N],[5,"move_items",E,"Moves at most `count` items from the `src` consumer to the…",N,[[[R[13]],[R[0]],[R[2],[R[0]]],[R[3]]],[R[0]]]],[11,R[4],E,R[5],0,[[["self"]],[R[0]]]],[11,R[6],E,R[7],0,[[["self"]],["bool"]]],[11,R[8],E,R[9],0,[[["self"]],["bool"]]],[11,"len",E,"The length of the data stored in the buffer",0,[[["self"]],[R[0]]]],[11,R[10],E,R[11],0,[[["self"]],[R[0]]]],[11,"access",E,"Gives immutable access to the elements contained by the…",0,[[["self"],[R[1]]]]],[11,"access_mut",E,"Gives mutable access to the elements contained by the ring…",0,[[["self"],[R[1]]]]],[11,"pop_access",E,"Allows to read from ring buffer memory directry.",0,[[["self"],["f"]],[R[0]]]],[11,"pop_copy",E,"Copies data from the ring buffer to the slice in…",0,[[["self"]],[R[0]]]],[11,"pop",E,"Removes latest element from the ring buffer and returns…",0,[[["self"]],[R[2]]]],[11,"pop_each",E,"Repeatedly calls the closure `f` passing elements removed…",0,[[["self"],[R[0]],[R[2],[R[0]]],["fnmut"]],[R[0]]]],[11,"for_each",E,"Iterate immutably over the elements contained by the ring…",0,[[["self"],["fnmut"]]]],[11,"for_each_mut",E,"Iterate mutably over the elements contained by the ring…",0,[[["self"],["fnmut"]]]],[11,"move_to",E,R[12],0,[[["self"],[R[0]],[R[2],[R[0]]],[R[3]]],[R[0]]]],[11,"pop_slice",E,"Removes first elements from the ring buffer and writes…",0,[[["self"]],[R[0]]]],[11,"write_into",E,"Removes at most first `count` bytes from the ring buffer…",0,[[["self"],[R[0]],[R[2],[R[0]]],["write"]],[[R[14],[R[0]]],[R[0]]]]],[11,R[4],E,R[5],1,[[["self"]],[R[0]]]],[11,R[6],E,R[7],1,[[["self"]],["bool"]]],[11,R[8],E,R[9],1,[[["self"]],["bool"]]],[11,"len",E,"The length of the data stored in the buffer.",1,[[["self"]],[R[0]]]],[11,R[10],E,R[11],1,[[["self"]],[R[0]]]],[11,"push_access",E,"Allows to write into ring buffer memory directry.",1,[[["self"],["f"]],[R[0]]]],[11,"push_copy",E,"Copies data from the slice to the ring buffer in…",1,[[["self"]],[R[0]]]],[11,"push",E,"Appends an element to the ring buffer. On failure returns…",1,[[["self"],[T]],[R[14]]]],[11,"push_each",E,"Repeatedly calls the closure `f` and pushes elements…",1,[[["self"],["fnmut"]],[R[0]]]],[11,"push_iter",E,"Appends elements from an iterator to the ring buffer.…",1,[[["self"],["i"]],[R[0]]]],[11,"move_from",E,R[12],1,[[["self"],[R[13]],[R[0]],[R[2],[R[0]]]],[R[0]]]],[11,"push_slice",E,"Appends elements from slice to the ring buffer. Elements…",1,[[["self"]],[R[0]]]],[11,"read_from",E,"Reads at most `count` bytes from `Read` instance and…",1,[[["self"],["read"],[R[0]],[R[2],[R[0]]]],[[R[14],[R[0]]],[R[0]]]]],[11,"new",E,"Creates a new instance of a ring buffer.",2,[[[R[0]]],["self"]]],[11,"split",E,"Splits ring buffer into producer and consumer.",2,[[]]],[11,R[4],E,R[5],2,[[["self"]],[R[0]]]],[11,R[6],E,R[7],2,[[["self"]],["bool"]]],[11,R[8],E,R[9],2,[[["self"]],["bool"]]],[11,"len",E,"The length of the data in the buffer.",2,[[["self"]],[R[0]]]],[11,R[10],E,R[11],2,[[["self"]],[R[0]]]],[11,"into",E,E,0,[[],[U]]],[11,"from",E,E,0,[[[T]],[T]]],[11,R[15],E,E,0,[[[U]],[R[14]]]],[11,R[16],E,E,0,[[],[R[14]]]],[11,R[17],E,E,0,[[["self"]],[T]]],[11,"borrow",E,E,0,[[["self"]],[T]]],[11,R[18],E,E,0,[[["self"]],["typeid"]]],[11,"into",E,E,1,[[],[U]]],[11,"from",E,E,1,[[[T]],[T]]],[11,R[15],E,E,1,[[[U]],[R[14]]]],[11,R[16],E,E,1,[[],[R[14]]]],[11,R[17],E,E,1,[[["self"]],[T]]],[11,"borrow",E,E,1,[[["self"]],[T]]],[11,R[18],E,E,1,[[["self"]],["typeid"]]],[11,"into",E,E,2,[[],[U]]],[11,"from",E,E,2,[[[T]],[T]]],[11,R[15],E,E,2,[[[U]],[R[14]]]],[11,R[16],E,E,2,[[],[R[14]]]],[11,R[17],E,E,2,[[["self"]],[T]]],[11,"borrow",E,E,2,[[["self"]],[T]]],[11,R[18],E,E,2,[[["self"]],["typeid"]]],[11,"drop",E,E,2,[[["self"]]]],[11,"read",E,E,0,[[["self"]],[[R[14],[R[0]]],[R[0]]]]],[11,"write",E,E,1,[[["self"]],[[R[14],[R[0]]],[R[0]]]]],[11,"flush",E,E,1,[[["self"]],[R[14]]]]],"p":[[3,R[19]],[3,R[20]],[3,R[21]]]};
initSearch(searchIndex);addSearchOptions(searchIndex);