use anyhow::Result;
use windows::core::{InParam, Interface, BSTR, GUID, HSTRING, PWSTR};
use windows::Win32::Security::PSECURITY_DESCRIPTOR;
use windows::Win32::System::Com::*;
use windows::Win32::System::TaskScheduler::*;

// see https://learn.microsoft.com/en-us/windows/win32/taskschd/daily-trigger-example--c---
// see https://learn.microsoft.com/ja-jp/windows/win32/taskschd/c-c-code-example-creating-a-task-using-newworkitem

const TASK_NAME: &'static str = "com.anatawa12.vrc-log-renamer";

pub(crate) fn register_task_manager() -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)?;

        let service: ITaskService = CoCreateInstance(
            &GUID::from_u128(0x0f87369f_a4e5_4cfc_bd3e_73e6154572dd), // CLSID_TaskScheduler as _,
            InParam::null(),
            CLSCTX_INPROC_SERVER,
        )?;

        service.Connect(
            InParam::null(),
            InParam::null(),
            InParam::null(),
            InParam::null(),
        )?;

        let root_folder: ITaskFolder = service.GetFolder(&r"\".into())?;

        // delete if exists
        root_folder.DeleteTask(&TASK_NAME.into(), 0).ok();

        let task: ITaskDefinition = service.NewTask(0)?;
        drop(service);

        task.RegistrationInfo()?.SetAuthor(&"anatawa12".into())?;

        let daily_trigger: IDailyTrigger = task
            .Triggers()?
            .Create(TASK_TRIGGER_DAILY)?
            .cast::<IDailyTrigger>()?;
        daily_trigger.SetId(&"Trigger1".into())?;
        daily_trigger.SetStartBoundary(&"2022-10-14T00:00:00".into())?;
        daily_trigger.SetDaysInterval(1)?;

        let action: IExecAction = task.Actions()?.Create(TASK_ACTION_EXEC)?.cast()?;
        action.SetPath(&std::env::current_exe()?.to_string_lossy().as_ref().into())?;
        action.SetArguments(&"scheduled".into())?;

        let _task: IRegisteredTask = root_folder.RegisterTaskDefinition(
            &TASK_NAME.into(),
            &task,
            TASK_CREATE_OR_UPDATE.0,
            &VARIANT::default(),
            &VARIANT::default(),
            TASK_LOGON_INTERACTIVE_TOKEN,
            &VARIANT::default(),
        )?;
    }
    Ok(())
}

pub(crate) fn unregister_task_manager() -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)?;

        let service: ITaskService = CoCreateInstance(
            &GUID::from_u128(0x0f87369f_a4e5_4cfc_bd3e_73e6154572dd), // CLSID_TaskScheduler as _,
            InParam::null(),
            CLSCTX_INPROC_SERVER,
        )?;

        service.Connect(
            InParam::null(),
            InParam::null(),
            InParam::null(),
            InParam::null(),
        )?;

        let root_folder: ITaskFolder = service.GetFolder(&r"\".into())?;

        // delete if exists
        root_folder.DeleteTask(&TASK_NAME.into(), 0).ok();
    }
    Ok(())
}
