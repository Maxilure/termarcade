using System;
using System.Diagnostics;
using System.IO;
using System.Windows.Forms;

static class TermArcadeLauncher
{
    [STAThread]
    static void Main()
    {
        string dir = AppDomain.CurrentDomain.BaseDirectory;
        string ps1 = Path.Combine(dir, "termarcade.ps1");

        if (!File.Exists(ps1))
        {
            MessageBox.Show(
                "termarcade.ps1 not found.\nMake sure TermArcade.cs is in the same folder as termarcade.ps1.",
                "TermArcade",
                MessageBoxButtons.OK,
                MessageBoxIcon.Error);
            return;
        }

        var psi = new ProcessStartInfo
        {
            FileName = "powershell.exe",
            Arguments = $"-ExecutionPolicy Bypass -NoProfile -File \"{ps1}\"",
            UseShellExecute = false,
        };

        try
        {
            using var proc = Process.Start(psi);
            proc?.WaitForExit();
        }
        catch (Exception ex)
        {
            MessageBox.Show($"Failed to launch TermArcade:\n{ex.Message}", "TermArcade", MessageBoxButtons.OK, MessageBoxIcon.Error);
        }
    }
}
