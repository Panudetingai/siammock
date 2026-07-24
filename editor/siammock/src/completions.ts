import * as vscode from "vscode";

const HTTP_METHODS = [
  "GET",
  "POST",
  "PUT",
  "PATCH",
  "DELETE",
  "HEAD",
  "OPTIONS",
];

const BODY_TYPES = [
  "string",
  "number",
  "boolean",
  "array",
  "object",
  "string[]",
  "number[]",
  "boolean[]",
  "string (required)",
  "number (required)",
];

const STATIC_PLACEHOLDERS: Array<{
  label: string;
  detail: string;
  icon: string;
}> = [
  { label: "uuid", detail: "Generate UUID v4", icon: "symbol-key" },
  { label: "timestamp", detail: "Current ISO timestamp", icon: "clock" },
  {
    label: "random_number",
    detail: "Random number 1-10000",
    icon: "symbol-numeric",
  },
  { label: "jwt_token", detail: "Sample JWT token", icon: "key" },
  {
    label: "random_string",
    detail: "Random alphanumeric string",
    icon: "symbol-string",
  },
  { label: "thai_name", detail: "Random Thai name", icon: "person" },
  { label: "en_name", detail: "Random English name", icon: "person" },
  { label: "email", detail: "Random example email", icon: "mail" },
  { label: "currency", detail: "Random currency code", icon: "symbol-event" },
  {
    label: "payment_method",
    detail: "Random payment method",
    icon: "credit-card",
  },
  {
    label: "payment_status",
    detail: "Random payment status",
    icon: "checklist",
  },
  { label: "status", detail: "Random status value", icon: "info" },
  {
    label: "index",
    detail: "Repeat loop index (0-based)",
    icon: "list-ordered",
  },
];

const ROUTE_KEYS = [
  { key: "path", detail: "Route path, e.g. /api/v1/users/:id" },
  { key: "method", detail: "HTTP method" },
  { key: "summary", detail: "Human-readable description" },
  { key: "request", detail: "Request matching spec" },
  { key: "response", detail: "Response template" },
  { key: "save", detail: "Response save specification" },
];

const REQUEST_KEYS = [
  { key: "headers", detail: "Expected request headers" },
  { key: "query_params", detail: "Expected query parameters" },
  { key: "body", detail: "Request body schema or examples" },
];

const SIAMMOCK_LANGUAGES: vscode.DocumentFilter[] = [
  { language: "jsonsi" },
  { language: "json", pattern: "**/mock/**" },
];

export function registerCompletions(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.languages.registerCompletionItemProvider(
      SIAMMOCK_LANGUAGES,
      {
        provideCompletionItems(document, position) {
          const linePrefix = document
            .lineAt(position.line)
            .text.slice(0, position.character);
          const fullText = document.getText();

          const placeholderItems = providePlaceholderCompletions(
            linePrefix,
            fullText,
            position,
            document,
          );
          if (placeholderItems) {
            return placeholderItems;
          }

          const methodItems = provideMethodCompletions(linePrefix);
          if (methodItems) {
            return methodItems;
          }

          const bodyTypeItems = provideBodyTypeCompletions(linePrefix);
          if (bodyTypeItems) {
            return bodyTypeItems;
          }

          return provideKeyCompletions(linePrefix);
        },
      },
      '"',
      ":",
      "{",
    ),
  );
}

function providePlaceholderCompletions(
  linePrefix: string,
  fullText: string,
  position: vscode.Position,
  document: vscode.TextDocument,
): vscode.CompletionItem[] | undefined {
  const braceMatch = linePrefix.match(/\{\{([a-zA-Z0-9_:]*)$/);
  if (!braceMatch && !linePrefix.endsWith("{{")) {
    return undefined;
  }

  const partial = braceMatch?.[1] ?? "";
  const context = extractRouteContext(fullText, document.offsetAt(position));
  const items: vscode.CompletionItem[] = [];

  for (const placeholder of STATIC_PLACEHOLDERS) {
    if (partial && !placeholder.label.startsWith(partial)) {
      continue;
    }

    items.push(
      makePlaceholderItem(
        placeholder.label,
        placeholder.detail,
        placeholder.icon,
      ),
    );
  }

  for (const param of context.pathParams) {
    const label = `param:${param}`;
    if (partial && !label.startsWith(partial)) {
      continue;
    }
    items.push(
      makePlaceholderItem(label, `Value from path parameter :${param}`, "link"),
    );
  }

  for (const field of context.bodyFields) {
    const label = `body:${field}`;
    if (partial && !label.startsWith(partial)) {
      continue;
    }
    items.push(
      makePlaceholderItem(
        label,
        `Value from request.body field "${field}"`,
        "inbox",
      ),
    );
  }

  items.push(
    makePlaceholderItem("index:1", "Loop index + offset", "list-ordered"),
    makePlaceholderItem("csv:file.csv:column", "CSV column value", "table"),
    makePlaceholderItem("csv_count:file.csv", "CSV row count", "graph"),
  );

  return items;
}

function makePlaceholderItem(
  label: string,
  detail: string,
  _iconId: string,
): vscode.CompletionItem {
  const item = new vscode.CompletionItem(
    label,
    vscode.CompletionItemKind.Variable,
  );
  item.detail = detail;
  item.documentation = `Inserts {{${label}}}`;
  item.insertText = label;
  item.sortText = `0_${label}`;
  return item;
}

function provideMethodCompletions(
  linePrefix: string,
): vscode.CompletionItem[] | undefined {
  if (!/"method"\s*:\s*"[^"]*$/.test(linePrefix)) {
    return undefined;
  }

  return HTTP_METHODS.map((method) => {
    const item = new vscode.CompletionItem(
      method,
      vscode.CompletionItemKind.EnumMember,
    );
    item.detail = "HTTP method";
    item.sortText = `1_${method}`;
    return item;
  });
}

function provideBodyTypeCompletions(
  linePrefix: string,
): vscode.CompletionItem[] | undefined {
  if (!/"body"\s*:\s*\{[^}]*"[^"]*"\s*:\s*"[^"]*$/.test(linePrefix)) {
    return undefined;
  }

  return BODY_TYPES.map((typeName) => {
    const item = new vscode.CompletionItem(
      typeName,
      vscode.CompletionItemKind.TypeParameter,
    );
    item.detail = "Request body type descriptor";
    item.sortText = `2_${typeName}`;
    return item;
  });
}

function provideKeyCompletions(
  linePrefix: string,
): vscode.CompletionItem[] | undefined {
  const trimmed = linePrefix.trimEnd();

  if (trimmed.endsWith("{") || trimmed.endsWith(",")) {
    const keys = [...ROUTE_KEYS, ...REQUEST_KEYS];
    return keys.map(({ key, detail }) => makeKeyItem(key, detail));
  }

  if (
    /"request"\s*:\s*\{\s*$/.test(linePrefix) ||
    /"request"\s*:\s*\{[^}]*,\s*$/.test(linePrefix)
  ) {
    return REQUEST_KEYS.map(({ key, detail }) => makeKeyItem(key, detail));
  }

  return undefined;
}

function makeKeyItem(key: string, detail: string): vscode.CompletionItem {
  const item = new vscode.CompletionItem(
    key,
    vscode.CompletionItemKind.Property,
  );
  item.detail = detail;
  item.insertText = `"${key}": `;
  item.sortText = `3_${key}`;
  return item;
}

interface RouteContext {
  pathParams: string[];
  bodyFields: string[];
}

function extractRouteContext(source: string, offset: number): RouteContext {
  const before = source.slice(0, offset);
  const routeChunk = before.slice(before.lastIndexOf("{"));

  const pathMatch = routeChunk.match(/"path"\s*:\s*"([^"]+)"/);
  const path = pathMatch?.[1] ?? "";
  const pathParams = [...path.matchAll(/:([a-zA-Z_][a-zA-Z0-9_]*)/g)].map(
    (match) => match[1],
  );

  const bodyFields: string[] = [];
  const bodyMatch = routeChunk.match(/"body"\s*:\s*\{([^}]*)$/s);
  if (bodyMatch) {
    for (const fieldMatch of bodyMatch[1].matchAll(
      /"([a-zA-Z_][a-zA-Z0-9_]*)"\s*:/g,
    )) {
      bodyFields.push(fieldMatch[1]);
    }
  }

  return { pathParams, bodyFields };
}
