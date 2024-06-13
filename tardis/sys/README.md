In this version, I have only sketched out the APIs of [*μC/OS*](https://micrium.atlassian.net/wiki/home) in [*syzlang*](https://github.com/google/syzkaller/blob/master/docs/syscall_descriptions.md), which is a necessary step to build a complete fuzzing system. Perhaps the following needs further elaboration:

1. More detailed pointer direction.

   A better definition of pointer direction(`in/out/inout`) can improve the quality of "system call" sequence generation. 

   For function parameters of pointer type, I try my best to determine their direction, and when in doubt, I use the universal type `in`. When generating and mutating parameters, the generator treats both `in` and `inout` type parameters in the same way, randomly assigning them initial values. And for the `out` types, the generator will directly set them to zero as they are not important.

   But for the member of structs whose type is the pointer, instead of careful processing and judgment, I give them the default type `in`, as the project is too big...

   There is also a special treatment. In *syzlang*, only simple types, that is, various variants of int, can be used as “resources”. *μC/OS* mostly uses complex structures to represent a structure, it is difficult to control the generation order in this way. Instead, we adopted a little trick: for the function that creates a certain [*task*](https://micrium.atlassian.net/wiki/spaces/osiiidoc/pages/131329/About+Task+Management), we set its parameter as `out`, and for other functions that manipulate tasks, its parameter is `in`.

2. More specialization.

   It is not difficult to imagine that instantiating some of the parameters of a system call can greatly improve the code coverage and increase the probability of finding vulnerabilities/bugs. However, to save time, I just described the basic APIs without instantiating any system calls.

3. Conditional compile

   Because each module is independent but affects each other, there are many conditional compilation options when describing a module. More annoyingly, these conditional compilation options are written into the struct!!! Similarly, to save time, instead of describing various combinations of structure member variables, we chose the default conditional compilation options, but enabled the `OS_CFG_DBG_EN` parameter.

   For "system calls" which are disabled, I described but commented out them. For example, 'os3/thread_local_storage.txt'.
   
4. More modules

   It only describes the core modules of an operating system, such as os, CPU, etc., but does not describe the huge peripheral parts.

5. More refined processing of parameter ranges

   For the parameter *opt*, the caller can specify two meanings through the "add". We use the [*flags*](https://github.com/google/syzkaller/blob/master/docs/syscall_descriptions.md#flagsenums) to list their candidate ranges one by one, but didn’t consider the added value.

6. More accurate resource flow
   
   For some resources that are difficult to distinguish, this version only defines the direction of the resource outflow in the function return value, but does not define the inflowing resource in the formal parameter.


