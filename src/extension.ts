import * as vscode from 'vscode';
import { execSync, exec } from 'child_process';
import * as path from 'path';
import * as fs from 'fs';

export function activate(context: vscode.ExtensionContext) {
    let disposable = vscode.commands.registerCommand('my-rust-tool.hostImage', async () => {
        
        // 1. Get the Image Path
        const fileUri = await vscode.window.showOpenDialog({
            canSelectMany: false,
            openLabel: 'Select Image to Host',
            filters: { 'Images': ['png', 'jpg', 'jpeg'] }
        });

        if (!fileUri || !fileUri[0]) return;
        const imagePath = fileUri[0].fsPath;

        // 2. Locate/Build the Rust Binary
        const isWindows = process.platform === 'win32';
        const binaryName = isWindows ? 'volt.exe' : 'volt';
        const projectRoot = context.extensionPath;
        const binaryPath = path.join(projectRoot, 'target', 'debug', binaryName);

        // Auto-build if the file is missing so you don't have to deal with "where is it"
        if (!fs.existsSync(binaryPath)) {
            vscode.window.showInformationMessage('Binary missing. Compiling Rust project...');
            try {
                execSync('cargo build', { cwd: projectRoot });
            } catch (e) {
                vscode.window.showErrorMessage('Cargo build failed. Make sure Rust is installed.');
                return;
            }
        }

        // 3. Confirm and Execute
        const confirm = await vscode.window.showInformationMessage(
            `Host this image? \n${imagePath}`, 
            { modal: true }, 
            'Yes'
        );

        if (confirm === 'Yes') {
            // We use double quotes around paths to handle spaces in folder names
            exec(`"${binaryPath}" "${imagePath}"`, (err, stdout, stderr) => {
                if (err) {
                    vscode.window.showErrorMessage(`Rust Error: ${stderr || err.message}`);
                    return;
                }
                vscode.window.showInformationMessage(`Response: ${stdout}`);
            });
        }
    });

    context.subscriptions.push(disposable);
}