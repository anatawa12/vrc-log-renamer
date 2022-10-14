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

        CoInitializeSecurity(
            PSECURITY_DESCRIPTOR::default(),
            -1,
            None,
            None,
            RPC_C_AUTHN_LEVEL_PKT_PRIVACY,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            None,
            EOLE_AUTHENTICATION_CAPABILITIES(0),
            None,
        )
            .unwrap();

        let service: ITaskService = CoCreateInstance(
            &GUID::from_u128(0x0f87369f_a4e5_4cfc_bd3e_73e6154572dd), // CLSID_TaskScheduler as _,
            InParam::null(),
            CLSCTX_INPROC_SERVER,
        )
            .unwrap();

        service
            .Connect(
                InParam::null(),
                InParam::null(),
                InParam::null(),
                InParam::null(),
            )
            .unwrap();

        let root_folder: ITaskFolder = service.GetFolder(&r"\".into()).unwrap();

        // delete if exists
        root_folder.DeleteTask(&TASK_NAME.into(), 0).ok();

        let task: ITaskDefinition = service.NewTask(0).unwrap();
        drop(service);

        task.RegistrationInfo()?.SetAuthor(&"anatawa12".into())?;

        let daily_trigger: IDailyTrigger = task
            .Triggers()
            .unwrap()
            .Create(TASK_TRIGGER_DAILY)
            .unwrap()
            .cast::<IDailyTrigger>()
            .unwrap();
        daily_trigger.SetId(&"Trigger1".into()).unwrap();
        daily_trigger
            .SetStartBoundary(&"2022-10-14T00:00:00".into())
            .unwrap();
        daily_trigger.SetDaysInterval(1).unwrap();

        let action: IExecAction = task
            .Actions()
            .unwrap()
            .Create(TASK_ACTION_EXEC)
            .unwrap()
            .cast()
            .unwrap();
        action
            .SetPath(
                &std::env::current_exe()
                    .unwrap()
                    .to_string_lossy()
                    .as_ref()
                    .into(),
            )
            .unwrap();
        action.SetArguments(&"scheduled".into()).unwrap();

        let _task: IRegisteredTask = root_folder
            .RegisterTaskDefinition(
                &TASK_NAME.into(),
                &task,
                TASK_CREATE_OR_UPDATE.0,
                &VARIANT::default(),
                &VARIANT::default(),
                TASK_LOGON_INTERACTIVE_TOKEN,
                &VARIANT::default(),
            )
            .unwrap();
    }
    Ok(())
}

pub(crate) fn unregister_task_manager() -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)?;

        CoInitializeSecurity(
            PSECURITY_DESCRIPTOR::default(),
            -1,
            None,
            None,
            RPC_C_AUTHN_LEVEL_PKT_PRIVACY,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            None,
            EOLE_AUTHENTICATION_CAPABILITIES(0),
            None,
        )
            .unwrap();

        let service: ITaskService = CoCreateInstance(
            &GUID::from_u128(0x0f87369f_a4e5_4cfc_bd3e_73e6154572dd), // CLSID_TaskScheduler as _,
            InParam::null(),
            CLSCTX_INPROC_SERVER,
        )
            .unwrap();

        service
            .Connect(
                InParam::null(),
                InParam::null(),
                InParam::null(),
                InParam::null(),
            )
            .unwrap();

        let root_folder: ITaskFolder = service.GetFolder(&r"\".into()).unwrap();

        // delete if exists
        root_folder.DeleteTask(&TASK_NAME.into(), 0).ok();
    }
    Ok(())
}
