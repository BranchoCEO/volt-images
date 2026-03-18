import * as vscode from 'vscode';
import { execSync, exec } from 'child_process';
import * as path from 'path';
import * as fs from 'fs';

export function activate(context: vscode.ExtensionContext) {
    let disposable = vscode.commands.registerCommand('volt.hostImage', async () => {
        
        const fileUri = await vscode.window.showOpenDialog({
            canSelectMany: false,
            openLabel: 'Select Image',
            filters: { 'Images': ['png', 'jpg', 'jpeg'] }
        });

        if (!fileUri || !fileUri[0]) return;
        const imagePath = fileUri[0].fsPath;

        const isWindows = process.platform === 'win32';
        const binaryName = isWindows ? 'volt.exe' : 'volt';
        const projectRoot = context.extensionPath;
        const binaryPath = path.join(projectRoot, 'target', 'debug', binaryName);

        if (!fs.existsSync(binaryPath)) {
            vscode.window.showInformationMessage('Compiling Volt...');
            try {
                execSync('cargo build', { cwd: projectRoot });
            } catch (e) {
                vscode.window.showErrorMessage('Cargo build failed.');
                return;
            }
        }

        const confirm = await vscode.window.showInformationMessage(
            `Generate pixel map for: \n${imagePath}`, 
            { modal: true }, 
            'Yes'
        );

        if (confirm === 'Yes') {
            const command = isWindows ? `"${binaryPath}" "${imagePath}"` : `${binaryPath} "${imagePath}"`;
            
            exec(command, { cwd: path.dirname(imagePath) }, (err, stdout, stderr) => {
                if (err) {
                    vscode.window.showErrorMessage(`Error: ${stderr || err.message}`);
                    return;
                }
                vscode.window.showInformationMessage(`${stdout}`);
            });
        }
    });

    context.subscriptions.push(disposable);
}
