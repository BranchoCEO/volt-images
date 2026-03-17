"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
const vscode = __importStar(require("vscode"));
const child_process_1 = require("child_process");
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
function activate(context) {
    let disposable = vscode.commands.registerCommand('my-rust-tool.hostImage', async () => {
        // 1. Get the Image Path
        const fileUri = await vscode.window.showOpenDialog({
            canSelectMany: false,
            openLabel: 'Select Image to Host',
            filters: { 'Images': ['png', 'jpg', 'jpeg'] }
        });
        if (!fileUri || !fileUri[0])
            return;
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
                (0, child_process_1.execSync)('cargo build', { cwd: projectRoot });
            }
            catch (e) {
                vscode.window.showErrorMessage('Cargo build failed. Make sure Rust is installed.');
                return;
            }
        }
        // 3. Confirm and Execute
        const confirm = await vscode.window.showInformationMessage(`Host this image? \n${imagePath}`, { modal: true }, 'Yes');
        if (confirm === 'Yes') {
            // We use double quotes around paths to handle spaces in folder names
            (0, child_process_1.exec)(`"${binaryPath}" "${imagePath}"`, (err, stdout, stderr) => {
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
//# sourceMappingURL=extension.js.map