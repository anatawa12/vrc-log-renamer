use std::mem::{size_of, zeroed};
use anyhow::{bail, Result};
use windows::core::{Interface, InParam, IUnknown, PWSTR, HSTRING, BSTR, GUID};
use windows::Win32::Foundation::HWND;
use windows::Win32::Security::Credentials::*;
use windows::Win32::Security::PSECURITY_DESCRIPTOR;
use windows::Win32::System::TaskScheduler::*;
use windows::Win32::System::Com::*;
use windows::Win32::System::Ole::VariantInit;

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
            None
        ).unwrap();

        let service: ITaskService = CoCreateInstance(
            &GUID::from_u128(0x0f87369f_a4e5_4cfc_bd3e_73e6154572dd), // CLSID_TaskScheduler as _,
            InParam::null(),
            CLSCTX_INPROC_SERVER,
        ).unwrap();

        service.Connect(InParam::null(), InParam::null(), InParam::null(), InParam::null()).unwrap();

        let root_folder: ITaskFolder = service.GetFolder(&r"\".into()).unwrap();

        // delete if exists
        root_folder.DeleteTask(&TASK_NAME.into(), 0).ok();

        let task: ITaskDefinition = service.NewTask(0).unwrap();
        drop(service);

        task.RegistrationInfo()?.SetAuthor(&"anatawa12".into())?;

        let daily_trigger: IDailyTrigger = task.Triggers().unwrap().Create(TASK_TRIGGER_DAILY).unwrap().cast::<IDailyTrigger>().unwrap();
        daily_trigger.SetId(&"Trigger1".into()).unwrap();
        daily_trigger.SetStartBoundary(&"2022-10-14T00:00:00".into()).unwrap();
        daily_trigger.SetDaysInterval(1).unwrap();

        let action: IExecAction = task.Actions().unwrap().Create(TASK_ACTION_EXEC).unwrap().cast().unwrap();
        action.SetPath(&std::env::current_exe().unwrap().to_string_lossy().as_ref().into()).unwrap();
        action.SetArguments(&"scheduled".into()).unwrap();

        let info: CREDUI_INFOW = CREDUI_INFOW {
            cbSize: size_of::<CREDUI_INFOW>() as _,
            hwndParent: HWND::default(),
            pszMessageText: (&HSTRING::from("Account info for task register:")).into(),
            pszCaptionText: (&HSTRING::from("Enter Account Information for Task Registration")).into(),
            hbmBanner: Default::default(),
        };

        let mut username = [0 as u16; CREDUI_MAX_USERNAME_LENGTH as usize];
        let mut password = [0 as u16; /*CREDUI_MAX_PASSWORD_LENGTH as usize*/256];

        let err = CredUIPromptForCredentialsW(
            Some(&info),
            &HSTRING::from(""),
            None,
            0,
            &mut username,
            &mut password,
            None,
            CREDUI_FLAGS_GENERIC_CREDENTIALS |  //  Flags
                CREDUI_FLAGS_ALWAYS_SHOW_UI |
                CREDUI_FLAGS_DO_NOT_PERSIST
        );
        if err != 0 {
            bail!("Did not get credentials: {err}");
        }

        let _task: IRegisteredTask = root_folder.RegisterTaskDefinition(
            &TASK_NAME.into(),
            &task,
            TASK_CREATE_OR_UPDATE.0,
            &variant(BSTR::from_raw(&username[0])),
            &variant(BSTR::from_raw(&password[0])),
            TASK_LOGON_PASSWORD,
            &variant(BSTR::from("")),
        ).unwrap();
    }
    Ok(())
}

// this leaks memory but it's small so I ignore this problem
fn variant(str: BSTR) -> VARIANT {
    unsafe {
        let mut value: VARIANT = zeroed();
        VariantInit(&mut value as _);
        (*value.Anonymous.Anonymous).vt = VT_BSTR;
        *(*value.Anonymous.Anonymous).Anonymous.bstrVal = str;
        value
    }
}
