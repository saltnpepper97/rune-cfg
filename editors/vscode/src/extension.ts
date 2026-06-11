import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const outputChannel = vscode.window.createOutputChannel("RUNE Language Server");
  context.subscriptions.push(outputChannel);

  const config = vscode.workspace.getConfiguration("rune");
  const command = config.get<string>("server.path", "rune-lsp");
  const args = config.get<string[]>("server.args", []);

  const serverOptions: ServerOptions = {
    command,
    args,
    options: {
      env: process.env,
    },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "rune" }],
    outputChannel,
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher("**/*.rune"),
    },
  };

  client = new LanguageClient(
    "rune-lsp",
    "RUNE Language Server",
    serverOptions,
    clientOptions,
  );

  context.subscriptions.push(client);
  await client.start();
}

export async function deactivate(): Promise<void> {
  await client?.stop();
  client = undefined;
}
