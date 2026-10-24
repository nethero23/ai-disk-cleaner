#![cfg(windows)]

use cleaner_core::system::{parse_run, run_script, WINDOWS_OLD};
use std::{fs, process::Command};

// All OS-facing operations are mocked. Parse the generated AST before running
// it, so a future direct executable/delete/registry call cannot escape the test.
fn simulate(initial: bool, remaining: bool, attributes: &str, code: i32) -> (bool, bool) {
    simulate_with_setup(initial, remaining, attributes, code, "")
}

fn simulate_with_setup(
    initial: bool,
    remaining: bool,
    attributes: &str,
    code: i32,
    setup: &str,
) -> (bool, bool) {
    let temp = tempfile::tempdir().unwrap();
    let out = temp.path().join("result.json");
    let script = run_script(&[WINDOWS_OLD.into()], &out);
    let boolean = |v| if v { "$true" } else { "$false" };
    let wrapper = format!(
        r#"
$ErrorActionPreference = 'Stop'
$env:SystemDrive = 'C:'
$env:SystemRoot = 'C:\Windows'
$global:present = {initial}
$global:started = $false
function Test-Path {{ param($LiteralPath, $ErrorAction) if ($LiteralPath -ne 'C:\Windows.old') {{ throw 'unexpected path' }}; $global:present }}
function Get-Item {{ param($LiteralPath, [switch]$Force, $ErrorAction) if ($LiteralPath -ne 'C:\Windows.old') {{ throw 'unexpected path' }}; [pscustomobject]@{{ Attributes = [IO.FileAttributes]::{attributes} }} }}
function Start-Process {{
    param($FilePath, $ArgumentList, $WindowStyle, [switch]$Wait, [switch]$PassThru, $ErrorAction)
    if ($FilePath -ne 'C:\Windows\System32\dism.exe' -or ($ArgumentList -join ' ') -ne '/Online /Remove-OSUninstall /NoRestart /English' -or !$Wait -or !$PassThru -or $WindowStyle -ne 'Hidden') {{ throw 'unexpected process' }}
    $global:started = $true
    $global:present = {remaining}
    [pscustomobject]@{{ ExitCode = {code} }}
}}
{setup}
$script = @'
{script}
'@
$tokens = $null; $errors = $null
$ast = [System.Management.Automation.Language.Parser]::ParseInput($script, [ref]$tokens, [ref]$errors)
if ($errors.Count) {{ throw 'script did not parse' }}
$allowed = @('Join-Path', 'Test-Path', 'Get-Item', 'Start-Process', 'ConvertTo-Json', 'Out-File')
foreach ($command in $ast.FindAll({{ param($node) $node -is [System.Management.Automation.Language.CommandAst] }}, $true)) {{
    if ($command.GetCommandName() -notin $allowed) {{ throw "unmocked command: $($command.GetCommandName())" }}
}}
& $ast.GetScriptBlock()
if ($global:started) {{ 'MOCK_PROCESS_STARTED' }}
"#,
        initial = boolean(initial),
        remaining = boolean(remaining),
    );
    let ps = temp.path().join("mock.ps1");
    fs::write(
        &ps,
        [b"\xef\xbb\xbf".as_slice(), wrapper.as_bytes()].concat(),
    )
    .unwrap();
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-File"])
        .arg(&ps)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = fs::read_to_string(out).unwrap();
    let result = parse_run(&[WINDOWS_OLD.into()], &text);
    let started = String::from_utf8_lossy(&output.stdout).contains("MOCK_PROCESS_STARTED");
    (result[0].ok, started)
}

#[test]
fn windows_old_native_script_requires_verified_removal_and_never_restarts() {
    assert_eq!(simulate(true, false, "Directory", 0), (true, true));
    assert_eq!(simulate(true, true, "Directory", 0), (false, true));
    assert_eq!(simulate(true, true, "Directory", 5), (false, true));
    assert_eq!(simulate(true, true, "Directory", 3010), (false, true));
    assert_eq!(simulate(false, false, "Directory", 0), (true, false));
    assert_eq!(simulate(true, true, "ReparsePoint", 0), (false, false));
    assert_eq!(simulate(true, true, "Normal", 0), (false, false));
}

#[test]
fn windows_old_native_script_environment_and_exceptions_fail_closed() {
    for setup in [
        "$env:SystemDrive = 'invalid'",
        "$env:SystemRoot = 'D:\\Windows'",
        "function Test-Path { throw 'mock metadata denied' }",
        "function Get-Item { throw 'mock entry denied' }",
        "function Start-Process { throw 'mock launch denied' }",
    ] {
        assert_eq!(
            simulate_with_setup(true, false, "Directory", 0, setup),
            (false, false),
            "{setup}"
        );
    }
    assert_eq!(simulate(true, false, "Directory", 3010), (true, true));
}
