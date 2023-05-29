// VRC Log Renamer - the tool to rename logs of VRChat to have date info
// Copyright (C) 2022 anatawa12
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use anyhow::Result;
use winsafe::prelude::*;
use winsafe::*;
use winsafe::co::TASK_ACTION_TYPE;

// see https://learn.microsoft.com/en-us/windows/win32/taskschd/daily-trigger-example--c---

const TASK_NAME: &'static str = "com.anatawa12.vrc-log-renamer";

pub(crate) fn register_task_manager() -> Result<()> {
    let _scope = CoInitializeEx(co::COINIT::MULTITHREADED);

    let service: ITaskService =
        CoCreateInstance(&co::CLSID::TaskScheduler, None, co::CLSCTX::INPROC_SERVER)?;

    service.Connect(None, None, None, None)?;

    let root_folder: ITaskFolder = service.GetFolder(&r"\")?;

    // delete if exists
    root_folder.DeleteTask(TASK_NAME).ok();

    let task: ITaskDefinition = service.NewTask()?;
    drop(service);

    task.get_RegistrationInfo()?.put_Author(&"anatawa12")?;

    let daily_trigger: IDailyTrigger = task
        .get_Triggers()?
        .Create(co::TASK_TRIGGER_TYPE2::DAILY)?
        .QueryInterface::<IDailyTrigger>()?;
    daily_trigger.put_Id(&"Trigger1")?;
    daily_trigger.put_StartBoundary(&"2022-10-14T00:00:00")?;
    daily_trigger.put_DaysInterval(1)?;

    let action: IExecAction = task.get_Actions()?.Create(TASK_ACTION_TYPE::EXEC)?.QueryInterface()?;
    action.put_Path(&std::env::current_exe()?.to_string_lossy().as_ref())?;
    action.put_Arguments(&"scheduled")?;

    let _task: IRegisteredTask = root_folder.RegisterTaskDefinition(
        Some(TASK_NAME),
        &task,
        co::TASK_CREATION::CREATE_OR_UPDATE,
        None,
        None,
        co::TASK_LOGON::INTERACTIVE_TOKEN,
        None,
    )?;
    Ok(())
}

pub(crate) fn unregister_task_manager() -> Result<()> {
    let _scope = CoInitializeEx(co::COINIT::MULTITHREADED);

    let service: ITaskService =
        CoCreateInstance(&co::CLSID::TaskScheduler, None, co::CLSCTX::INPROC_SERVER)?;

    service.Connect(None, None, None, None)?;

    let root_folder: ITaskFolder = service.GetFolder(r"\")?;

    // delete if exists
    root_folder.DeleteTask(TASK_NAME).ok();
    Ok(())
}
