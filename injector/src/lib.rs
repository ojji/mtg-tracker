mod assembler;
mod process;
mod utils;

use assembler::Assembler;
use process::{processes, ExportedFunction, Module, Process};
use std::{collections::HashMap, convert::TryInto, error::Error, fs, path::Path};

pub struct Mtga {
    process: Process,
    mono_functions: HashMap<RequiredFunction, process::ExportedFunction>,
}

impl Mtga {
    pub fn new() -> Result<Mtga, Box<dyn Error>> {
        let process = processes()
            .find(|process| process.name() == "MTGA.exe")
            .ok_or("Could not find MTGA.exe")?;

        let mono_module = Mtga::find_mono_module(&process)?;

        let mono_functions = Mtga::find_required_mono_functions(&process, &mono_module)?;

        Ok(Mtga {
            process,
            mono_functions,
        })
    }

    pub fn inject_tracker<P>(&self, collector_path: P) -> Result<(), Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let collector_data = fs::read(collector_path)?;
        let root_domain_ptr = self.get_root_domain()?;
        let load_image_ptr = self.create_mono_image_from_data(&collector_data, root_domain_ptr)?;

        let datacollector_assembly_ptr =
            self.create_mono_assembly_from_image(root_domain_ptr, load_image_ptr)?;
        self.close_load_image(root_domain_ptr, load_image_ptr)?;

        let assembly_image_ptr =
            self.get_image_from_assembly(root_domain_ptr, datacollector_assembly_ptr)?;

        let loader_class_ptr = self.get_class_from_image(
            root_domain_ptr,
            assembly_image_ptr,
            "mtga_datacollector",
            "Loader",
        )?;

        let load_method_ptr =
            self.get_method_from_class(root_domain_ptr, loader_class_ptr, "Load", 0)?;

        self.runtime_invoke(root_domain_ptr, load_method_ptr, None)?;

        Ok(())
    }

    pub fn inject_dumper<P>(&self, collector_path: P) -> Result<(), Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let collector_data = fs::read(collector_path)?;
        let root_domain_ptr = self.get_root_domain()?;
        let load_image_ptr = self.create_mono_image_from_data(&collector_data, root_domain_ptr)?;

        let datacollector_assembly_ptr =
            self.create_mono_assembly_from_image(root_domain_ptr, load_image_ptr)?;
        self.close_load_image(root_domain_ptr, load_image_ptr)?;

        let assembly_image_ptr =
            self.get_image_from_assembly(root_domain_ptr, datacollector_assembly_ptr)?;

        let loader_class_ptr = self.get_class_from_image(
            root_domain_ptr,
            assembly_image_ptr,
            "mtga_datacollector",
            "Loader",
        )?;

        let load_method_ptr =
            self.get_method_from_class(root_domain_ptr, loader_class_ptr, "LoadDumper", 1)?;

        let memory_manager = self.process.get_memory_manager()?;

        let dump_directory = Path::new("./assets/").canonicalize()?;
        let dump_directory = dump_directory
            .to_str()
            .ok_or("cannot convert directory path to string")?;

        let dump_dir_monostring_ptr = self.create_mono_string(root_domain_ptr, dump_directory)?;
        let params_ptr = memory_manager.allocate_and_write(&dump_dir_monostring_ptr.to_le_bytes())?;

        self.runtime_invoke(root_domain_ptr, load_method_ptr, Some(params_ptr))?;

        Ok(())
    }

    fn find_mono_module(process: &Process) -> Result<Module, Box<dyn Error>> {
        let mono_module = process
            .modules()
            .find(|module| module.name().starts_with("mono-"))
            .ok_or("Could not find the mono module")?;

        Ok(mono_module)
    }

    fn find_required_mono_functions(
        process: &Process,
        mono_module: &Module,
    ) -> Result<HashMap<RequiredFunction, ExportedFunction>, Box<dyn Error>> {
        let mut required_functions = HashMap::new();

        for exported_fn in process.get_exports_for_module(mono_module)?.into_iter() {
            if let Ok(required_fn) = RequiredFunction::try_from(exported_fn.name()) {
                required_functions.entry(required_fn).or_insert(exported_fn);
            }
        }

        Ok(required_functions)
    }

    fn get_root_domain(&self) -> Result<usize, Box<dyn Error>> {
        let memory_manager = self.process.get_memory_manager()?;

        // functions
        let get_root_domain_fn = self
            .mono_functions
            .get(&RequiredFunction::MonoGetRootDomain)
            .ok_or("Could not find get_root_domain()")?;

        // params

        // out params
        let ret_val_ptr = memory_manager.allocate_and_write(&[0; std::mem::size_of::<usize>()])?;

        /*
        MONO_API MonoDomain* mono_get_root_domain (void);
        */
        let mut assembler = Assembler::new();
        assembler.sub_rsp(0x28);
        assembler.mov_rax(get_root_domain_fn.address().try_into()?);
        assembler.call_rax();
        assembler.mov_rax_to(ret_val_ptr);
        assembler.add_rsp(0x28);
        assembler.ret();

        // execute and return
        let code_ptr = memory_manager.allocate_and_write(assembler.data())?;
        self.process.execute(code_ptr)?;

        let mut get_root_domain_ptr = 0_usize;
        memory_manager.read_from_address(ret_val_ptr, &mut get_root_domain_ptr)?;
        Ok(get_root_domain_ptr)
    }

    fn create_mono_image_from_data(
        &self,
        assembly_to_inject: &[u8],
        root_domain_ptr: usize,
    ) -> Result<usize, Box<dyn Error>> {
        let memory_manager = self.process.get_memory_manager()?;

        // functions
        let mono_thread_attach_fn = self
            .mono_functions
            .get(&RequiredFunction::MonoThreadAttach)
            .ok_or("Could not find mono_thread_attach()")?;

        let mono_image_open_from_data_fn = self
            .mono_functions
            .get(&RequiredFunction::MonoImageOpenFromData)
            .ok_or("Could not find mono_image_open_from_data()")?;

        // params
        let assembly_data_ptr = memory_manager.allocate_and_write(assembly_to_inject)?;

        /*
        typedef enum {
                MONO_IMAGE_OK,
                MONO_IMAGE_ERROR_ERRNO,
                MONO_IMAGE_MISSING_ASSEMBLYREF,
                MONO_IMAGE_IMAGE_INVALID
        } MonoImageOpenStatus;
        */

        let image_open_status_ptr =
            memory_manager.allocate_and_write(&[0; std::mem::size_of::<u32>()])?;

        // out params
        let ret_val_ptr = memory_manager.allocate_and_write(&[0; std::mem::size_of::<usize>()])?;

        // assembly
        let mut assembler = Assembler::new();
        assembler.sub_rsp(0x28);

        // register our thread with mono
        assembler.mov_rax(mono_thread_attach_fn.address().try_into()?);
        assembler.mov_rcx(root_domain_ptr.try_into()?);
        assembler.call_rax();

        /*
        MONO_API MONO_RT_EXTERNAL_ONLY
        MonoImage* mono_image_open_from_data (char *data, uint32_t data_len, mono_bool need_copy,
                                              MonoImageOpenStatus *status);
        */

        assembler.mov_rax(mono_image_open_from_data_fn.address().try_into()?);
        assembler.mov_rcx(assembly_data_ptr.try_into()?);
        assembler.mov_rdx(assembly_to_inject.len().try_into()?);
        assembler.mov_r8(1); // make a copy of the image so the assembly will have its own image containing the running code
        assembler.mov_r9(image_open_status_ptr.try_into()?);
        assembler.call_rax();
        assembler.mov_rax_to(ret_val_ptr);
        assembler.add_rsp(0x28);
        assembler.ret();

        // execute and return
        let code_ptr = memory_manager.allocate_and_write(assembler.data())?;
        self.process.execute(code_ptr)?;

        let mut open_status_result = 0x0_u32;
        memory_manager.read_from_address(image_open_status_ptr, &mut open_status_result)?;
        if open_status_result != 0 {
            return Err("Could not create image from data!".into());
        }

        let mut image_ptr = 0_usize;
        memory_manager.read_from_address(ret_val_ptr, &mut image_ptr)?;
        Ok(image_ptr)
    }

    fn create_mono_assembly_from_image(
        &self,
        root_domain_ptr: usize,
        image_ptr: usize,
    ) -> Result<usize, Box<dyn Error>> {
        let memory_manager = self.process.get_memory_manager()?;

        // functions
        let mono_thread_attach_fn = self
            .mono_functions
            .get(&RequiredFunction::MonoThreadAttach)
            .ok_or("Could not find mono_thread_attach()")?;

        let mono_assembly_load_from_fn = self
            .mono_functions
            .get(&RequiredFunction::MonoAssemblyLoadFrom)
            .ok_or("Could not find mono_assembly_load_from()")?;

        // params
        let mut assembly_name = String::from("tracker-injector-");
        assembly_name.push_str(utils::Guid::rand().to_string().as_str());
        assembly_name.push('\0');
        let assembly_name_ptr = memory_manager.allocate_and_write(assembly_name.as_bytes())?;

        // out params
        let ret_val_ptr =
            memory_manager.allocate_and_write(&[0_u8; std::mem::size_of::<usize>()])?;

        let open_status_ptr =
            memory_manager.allocate_and_write(&[0_u8; std::mem::size_of::<u32>()])?;

        let mut assembler = Assembler::new();
        assembler.sub_rsp(0x28);

        // register our thread with mono
        assembler.mov_rax(mono_thread_attach_fn.address().try_into()?);
        assembler.mov_rcx(root_domain_ptr.try_into()?);
        assembler.call_rax();

        /*
        MONO_API MONO_RT_EXTERNAL_ONLY
        MonoAssembly* mono_assembly_load_from (MonoImage *image, const char *fname,
                                               MonoImageOpenStatus *status);
         */
        assembler.mov_rax(mono_assembly_load_from_fn.address().try_into()?);
        assembler.mov_rcx(image_ptr.try_into()?);
        assembler.mov_rdx(assembly_name_ptr.try_into()?);
        assembler.mov_r8(open_status_ptr.try_into()?);
        assembler.call_rax();
        assembler.mov_rax_to(ret_val_ptr);

        assembler.add_rsp(0x28);
        assembler.ret();

        // execute
        let code_ptr = memory_manager.allocate_and_write(assembler.data())?;
        self.process.execute(code_ptr)?;

        // check image_open_status and return assembly_pointer
        let mut image_open_status = 0_u32;
        memory_manager.read_from_address(open_status_ptr, &mut image_open_status)?;
        if image_open_status != 0 {
            return Err(format!(
                "mono_assembly_load_from() returned with error: {}",
                image_open_status
            )
            .into());
        }

        let mut assembly_ptr = 0_usize;
        memory_manager.read_from_address(ret_val_ptr, &mut assembly_ptr)?;

        Ok(assembly_ptr)
    }

    fn get_class_from_image(
        &self,
        root_domain_ptr: usize,
        image_ptr: usize,
        namespace: &str,
        class_name: &str,
    ) -> Result<usize, Box<dyn Error>> {
        let memory_manager = self.process.get_memory_manager()?;

        // functions
        let thread_attach_fn = self
            .mono_functions
            .get(&RequiredFunction::MonoThreadAttach)
            .ok_or("Could not find mono_thread_attach()")?;

        let mono_class_from_name_fn = self
            .mono_functions
            .get(&RequiredFunction::MonoClassFromName)
            .ok_or("Could not find mono_class_from_name()")?;

        // params
        let mut namespace = Vec::from(namespace.as_bytes());
        namespace.push(0);
        let namespace_ptr = memory_manager.allocate_and_write(&namespace)?;

        let mut class_name = Vec::from(class_name.as_bytes());
        class_name.push(0);
        let class_name_ptr = memory_manager.allocate_and_write(&class_name)?;

        // out params
        let ret_val_ptr =
            memory_manager.allocate_and_write(&[0_u8; std::mem::size_of::<usize>()])?;

        // assembly
        let mut assembler = Assembler::new();
        assembler.sub_rsp(0x28);

        assembler.mov_rax(thread_attach_fn.address().try_into()?);
        assembler.mov_rcx(root_domain_ptr.try_into()?);
        assembler.call_rax();

        /*
        MONO_API MONO_RT_EXTERNAL_ONLY
        MonoClass* mono_class_from_name (MonoImage *image, const char* name_space, const char *name);
        */
        assembler.mov_rax(mono_class_from_name_fn.address().try_into()?);
        assembler.mov_rcx(image_ptr.try_into()?);
        assembler.mov_rdx(namespace_ptr.try_into()?);
        assembler.mov_r8(class_name_ptr.try_into()?);
        assembler.call_rax();
        assembler.mov_rax_to(ret_val_ptr);

        assembler.add_rsp(0x28);
        assembler.ret();

        // execute and return
        let code_ptr = memory_manager.allocate_and_write(assembler.data())?;
        self.process.execute(code_ptr)?;

        let mut mono_class_ptr = 0_usize;
        memory_manager.read_from_address(ret_val_ptr, &mut mono_class_ptr)?;
        Ok(mono_class_ptr)
    }

    fn get_method_from_class(
        &self,
        root_domain_ptr: usize,
        class_ptr: usize,
        method_name: &str,
        num_params: u64,
    ) -> Result<usize, Box<dyn Error>> {
        let memory_manager = self.process.get_memory_manager()?;

        // functions
        let mono_thread_attach_fn = self
            .mono_functions
            .get(&RequiredFunction::MonoThreadAttach)
            .ok_or("Could not find mono_thread_attach()")?;

        let mono_class_get_method_from_name_fn = self
            .mono_functions
            .get(&RequiredFunction::MonoClassGetMethodFromName)
            .ok_or("Could not find mono_class_get_method_from_name()")?;

        // params
        let mut method_name = Vec::from(method_name.as_bytes());
        method_name.push(0);
        let method_name_ptr = memory_manager.allocate_and_write(&method_name)?;

        // out params
        let ret_val_ptr =
            memory_manager.allocate_and_write(&[0_u8; std::mem::size_of::<usize>()])?;

        // assembly
        let mut assembler = Assembler::new();
        assembler.sub_rsp(0x28);

        // register our thread with mono
        assembler.mov_rax(mono_thread_attach_fn.address().try_into()?);
        assembler.mov_rcx(root_domain_ptr.try_into()?);
        assembler.call_rax();

        /*
        MONO_API MONO_RT_EXTERNAL_ONLY
        MonoMethod* mono_class_get_method_from_name (MonoClass *klass, const char *name, int param_count);
        */
        assembler.mov_rax(mono_class_get_method_from_name_fn.address().try_into()?);
        assembler.mov_rcx(class_ptr.try_into()?);
        assembler.mov_rdx(method_name_ptr.try_into()?);
        assembler.mov_r8(num_params);
        assembler.call_rax();
        assembler.mov_rax_to(ret_val_ptr);

        assembler.add_rsp(0x28);
        assembler.ret();

        // execute and return
        let code_ptr = memory_manager.allocate_and_write(assembler.data())?;
        self.process.execute(code_ptr)?;

        let mut method_ptr = 0_usize;
        memory_manager.read_from_address(ret_val_ptr, &mut method_ptr)?;

        Ok(method_ptr)
    }

    fn runtime_invoke(
        &self,
        root_domain_ptr: usize,
        method_ptr: usize,
        params_ptr: Option<usize>,
    ) -> Result<(), Box<dyn Error>> {
        let memory_manager = self.process.get_memory_manager()?;

        // functions
        let mono_thread_attach_fn = self
            .mono_functions
            .get(&RequiredFunction::MonoThreadAttach)
            .ok_or("Could not find mono_thread_attach()")?;

        let mono_runtime_invoke_fn = self
            .mono_functions
            .get(&RequiredFunction::MonoRuntimeInvoke)
            .ok_or("Could not find mono_runtime_invoke()")?;

        // params
        let exception_object_ptr =
            memory_manager.allocate_and_write(&[0_u8; std::mem::size_of::<usize>()])?;

        // assembly
        let mut assembler = Assembler::new();
        assembler.sub_rsp(0x28);

        // register our thread with mono
        assembler.mov_rax(mono_thread_attach_fn.address().try_into()?);
        assembler.mov_rcx(root_domain_ptr.try_into()?);
        assembler.call_rax();

        /*
        MONO_API MONO_RT_EXTERNAL_ONLY
        MonoObject* mono_runtime_invoke (MonoMethod *method, void *obj, void **params, MonoObject **exc);
        */
        assembler.mov_rax(mono_runtime_invoke_fn.address().try_into()?);
        assembler.mov_rcx(method_ptr.try_into()?);
        assembler.mov_rdx(0);
        assembler.mov_r8(params_ptr.unwrap_or(0).try_into()?);
        assembler.mov_r9(exception_object_ptr.try_into()?);
        assembler.call_rax();

        assembler.add_rsp(0x28);
        assembler.ret();

        // execute and return
        let code_ptr = memory_manager.allocate_and_write(assembler.data())?;
        self.process.execute(code_ptr)?;

        let mut exception_object = 0_usize;
        // check for exceptions
        memory_manager.read_from_address(exception_object_ptr, &mut exception_object)?;

        if exception_object != 0 {
            Err("Runtime invocation failed with an exception!".into())
        } else {
            Ok(())
        }
    }

    fn get_image_from_assembly(
        &self,
        root_domain_ptr: usize,
        assembly_ptr: usize,
    ) -> Result<usize, Box<dyn Error>> {
        let memory_manager = self.process.get_memory_manager()?;

        // functions
        let mono_thread_attach_fn = self
            .mono_functions
            .get(&RequiredFunction::MonoThreadAttach)
            .ok_or("Could not find mono_thread_attach()")?;

        let mono_assembly_get_image_fn = self
            .mono_functions
            .get(&RequiredFunction::MonoAssemblyGetImage)
            .ok_or("Could not find mono_assembly_get_image()")?;

        // out params
        let ret_val_ptr =
            memory_manager.allocate_and_write(&[0_u8; std::mem::size_of::<usize>()])?;

        // assembly
        let mut assembler = Assembler::new();
        assembler.sub_rsp(0x28);

        // register our thread with mono
        assembler.mov_rax(mono_thread_attach_fn.address().try_into()?);
        assembler.mov_rcx(root_domain_ptr.try_into()?);
        assembler.call_rax();

        /*
        MONO_API MONO_RT_EXTERNAL_ONLY
        MonoImage* mono_assembly_get_image (MonoAssembly *assembly);
        */
        assembler.mov_rax(mono_assembly_get_image_fn.address().try_into()?);
        assembler.mov_rcx(assembly_ptr.try_into()?);
        assembler.call_rax();
        assembler.mov_rax_to(ret_val_ptr);

        assembler.add_rsp(0x28);
        assembler.ret();

        // execute and return
        let code_ptr = memory_manager.allocate_and_write(assembler.data())?;
        self.process.execute(code_ptr)?;

        let mut image_ptr = 0_usize;
        memory_manager.read_from_address(ret_val_ptr, &mut image_ptr)?;

        Ok(image_ptr)
    }

    fn close_load_image(
        &self,
        root_domain_ptr: usize,
        load_image_ptr: usize,
    ) -> Result<(), Box<dyn Error>> {
        let memory_manager = self.process.get_memory_manager()?;

        // functions
        let mono_thread_attach_fn = self
            .mono_functions
            .get(&RequiredFunction::MonoThreadAttach)
            .ok_or("Could not find mono_thread_attach()")?;

        let mono_image_close_fn = self
            .mono_functions
            .get(&RequiredFunction::MonoImageClose)
            .ok_or("Could not find mono_image_close()")?;

        // assembly
        let mut assembler = Assembler::new();
        assembler.sub_rsp(0x28);

        // register our thread with mono
        assembler.mov_rax(mono_thread_attach_fn.address().try_into()?);
        assembler.mov_rcx(root_domain_ptr.try_into()?);
        assembler.call_rax();

        /*
        MONO_API void mono_image_close (MonoImage *image);
        */
        assembler.mov_rax(mono_image_close_fn.address().try_into()?);
        assembler.mov_rcx(load_image_ptr.try_into()?);
        assembler.call_rax();

        assembler.add_rsp(0x28);
        assembler.ret();

        // execute and return
        let code_ptr = memory_manager.allocate_and_write(assembler.data())?;
        self.process.execute(code_ptr)?;
        Ok(())
    }

    fn create_mono_string(&self, root_domain_ptr: usize, s: &str) -> Result<usize, Box<dyn Error>> {
        let memory_manager = self.process.get_memory_manager()?;

        // functions
        let mono_thread_attach_fn = self
            .mono_functions
            .get(&RequiredFunction::MonoThreadAttach)
            .ok_or("Could not find mono_thread_attach()")?;

        let mono_string_new_len_fn = self
            .mono_functions
            .get(&RequiredFunction::MonoStringNewLen)
            .ok_or("Could not find mono_string_new_len()")?;

        // params
        let s = Vec::from(s.as_bytes());
        let s_ptr = memory_manager.allocate_and_write(&s)?;

        // output
        let ret_val = memory_manager.allocate_and_write(&[0_u8; std::mem::size_of::<usize>()])?;

        // assembly
        let mut assembler = Assembler::new();
        assembler.sub_rsp(0x28);

        // register our thread with mono
        assembler.mov_rax(mono_thread_attach_fn.address().try_into()?);
        assembler.mov_rcx(root_domain_ptr.try_into()?);
        assembler.call_rax();

        /*
        MONO_API MONO_RT_EXTERNAL_ONLY
        MonoString* mono_string_new_len (MonoDomain *domain, const char *text, unsigned int length);
        */
        assembler.mov_rax(mono_string_new_len_fn.address().try_into()?);
        assembler.mov_rcx(root_domain_ptr.try_into()?);
        assembler.mov_rdx(s_ptr.try_into()?);
        assembler.mov_r8(s.len().try_into()?);
        assembler.call_rax();
        assembler.mov_rax_to(ret_val);
        assembler.add_rsp(0x28);
        assembler.ret();

        // execute and return
        let code_ptr = memory_manager.allocate_and_write(assembler.data())?;
        self.process.execute(code_ptr)?;

        let mut string_ptr = 0_usize;
        memory_manager.read_from_address(ret_val, &mut string_ptr)?;
        Ok(string_ptr)
    }
}

#[derive(Hash, PartialEq, Eq)]
enum RequiredFunction {
    MonoGetRootDomain,
    MonoThreadAttach,
    MonoImageOpenFromData,
    MonoAssemblyLoadFrom,
    MonoClassFromName,
    MonoClassGetMethodFromName,
    MonoRuntimeInvoke,
    MonoAssemblyGetImage,
    MonoImageClose,
    MonoStringNewLen,
}

impl TryFrom<&str> for RequiredFunction {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "mono_get_root_domain" => Ok(RequiredFunction::MonoGetRootDomain),
            "mono_thread_attach" => Ok(RequiredFunction::MonoThreadAttach),
            "mono_image_open_from_data" => Ok(RequiredFunction::MonoImageOpenFromData),
            "mono_assembly_load_from" => Ok(RequiredFunction::MonoAssemblyLoadFrom),
            "mono_class_from_name" => Ok(RequiredFunction::MonoClassFromName),
            "mono_class_get_method_from_name" => Ok(RequiredFunction::MonoClassGetMethodFromName),
            "mono_runtime_invoke" => Ok(RequiredFunction::MonoRuntimeInvoke),
            "mono_assembly_get_image" => Ok(RequiredFunction::MonoAssemblyGetImage),
            "mono_image_close" => Ok(RequiredFunction::MonoImageClose),
            "mono_string_new_len" => Ok(RequiredFunction::MonoStringNewLen),
            _ => Err("Could not match function to any of the required ones"),
        }
    }
}
