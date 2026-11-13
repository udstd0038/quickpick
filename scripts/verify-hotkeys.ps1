param([int]$ProcessId = 0, [switch]$Legacy)
$ErrorActionPreference = 'Stop'
if ($Legacy) { & powershell -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'verify-hotkeys-tauri.ps1'); exit $LASTEXITCODE }
$process = if ($ProcessId) { Get-Process -Id $ProcessId } else { Get-Process -Name quickpick -ErrorAction Stop | Select-Object -First 1 }
$start = [Diagnostics.ProcessStartInfo]::new()
$start.FileName = $process.Path
$start.Arguments = '--hotkey-bindings'
$start.UseShellExecute = $false
$start.CreateNoWindow = $true
$start.RedirectStandardOutput = $true
$start.StandardOutputEncoding = [Text.UTF8Encoding]::new($false)
$reportProcess = [Diagnostics.Process]::Start($start)
$report = $reportProcess.StandardOutput.ReadToEnd() | ConvertFrom-Json
$reportProcess.WaitForExit()
if ($reportProcess.ExitCode -ne 0 -or -not $report.bindings) { throw '需要支持 --hotkey-bindings 的 GPUI 候选程序。旧版使用 -Legacy。' }
Add-Type -ReferencedAssemblies System.Windows.Forms @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Windows.Forms;
public static class QuickPickNativeVerifier {
 [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort vk, scan; public uint flags, time; public IntPtr extra; }
 [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int x,y; public uint mouseData,flags,time; public IntPtr extra; }
 [StructLayout(LayoutKind.Explicit)] public struct UNION { [FieldOffset(0)] public KEYBDINPUT key; [FieldOffset(0)] public MOUSEINPUT mouse; }
 [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public UNION data; }
 [DllImport("user32.dll",SetLastError=true)] static extern uint SendInput(uint n, INPUT[] inputs, int size);
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
 public delegate bool Visitor(IntPtr h,IntPtr p);
 [DllImport("user32.dll")] static extern bool EnumWindows(Visitor visitor,IntPtr p);
 [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h,out uint pid);
 [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h,System.Text.StringBuilder text,int size);
 public static string[] Titles(uint pid) { var titles=new List<string>(); EnumWindows((h,p)=>{uint owner;GetWindowThreadProcessId(h,out owner);if(owner==pid&&IsWindowVisible(h)){var text=new System.Text.StringBuilder(512);GetWindowText(h,text,512);titles.Add(text.ToString());}return true;},IntPtr.Zero);return titles.ToArray(); }
 public static void Keys(int[] codes) { var inputs=new List<INPUT>();foreach(int key in codes){inputs.Add(new INPUT{type=1,data=new UNION{key=new KEYBDINPUT{vk=(ushort)key}}});}for(int i=codes.Length-1;i>=0;i--){inputs.Add(new INPUT{type=1,data=new UNION{key=new KEYBDINPUT{vk=(ushort)codes[i],flags=2}}});}var array=inputs.ToArray();if(SendInput((uint)array.Length,array,Marshal.SizeOf(typeof(INPUT)))!=array.Length)throw new Exception("SendInput failed"); }
 public static Form Fixture() { var form=new Form{Text="QuickPick hotkey verification",Width=480,Height=220};var text=new TextBox{Multiline=true,Dock=DockStyle.Fill,Text="QuickPick synthetic selection fixture"};form.Controls.Add(text);form.Show();text.Focus();text.SelectAll();Application.DoEvents();SetForegroundWindow(form.Handle);return form; }
 public static void Pump() { Application.DoEvents(); }
}
'@
$fixture = [QuickPickNativeVerifier]::Fixture()
try {
    foreach ($role in @('settings','selection','input','screenshot')) {
        [QuickPickNativeVerifier]::SetForegroundWindow($fixture.Handle) | Out-Null
        [QuickPickNativeVerifier]::Pump()
        Start-Sleep -Milliseconds 250
        $binding = $report.bindings | Where-Object role -eq $role
        if (-not $binding -or -not $binding.press) { throw "无法读取 $role 快捷键" }
        [QuickPickNativeVerifier]::Keys([int[]]$binding.press)
        $expected = $report.titles.$role
        $deadline = [DateTime]::UtcNow.AddSeconds(5)
        do {
            [QuickPickNativeVerifier]::Pump()
            $titles = [QuickPickNativeVerifier]::Titles([uint32]$process.Id)
            if ($titles -contains $expected) { break }
            Start-Sleep -Milliseconds 50
        } while ([DateTime]::UtcNow -lt $deadline)
        if ($titles -notcontains $expected) { throw "$role 快捷键未显示预期窗口；请检查占用、权限和焦点。" }
        Write-Host "$role 路由和窗口显示通过"
        if ($role -eq 'settings') {
            [QuickPickNativeVerifier]::SetForegroundWindow($fixture.Handle) | Out-Null
        } else {
            [QuickPickNativeVerifier]::Keys([int[]]@(0x1b))
            Start-Sleep -Milliseconds 200
            if ([QuickPickNativeVerifier]::Titles([uint32]$process.Id) -contains $expected) { throw "$role Esc 关闭未完成" }
        }
    }
    Write-Host '检查完成：仅使用合成测试文本，无 AI 上传。真实跨应用选区、剪贴板恢复、DPI 和截图像素仍需按验收矩阵确认。'
} finally {
    $fixture.Close()
    $fixture.Dispose()
}
