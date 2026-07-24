# SiamMock

High-speed Mock API server ที่เขียน config ด้วย JSON อย่างเดียว — ไม่ต้องรอ backend จริง

สร้างด้วย Rust (Axum + Tokio) เพื่อ performance สูง รองรับ dynamic response, request validation และ mock data สำเร็จรูป

---

## Features

- **Config-driven routes** — กำหนด path, method, request/response ในไฟล์ JSON
- **Dynamic placeholders** — `{{uuid}}`, `{{body:field}}`, `{{param:id}}` และอื่นๆ
- **Mock data generators** — ชื่อไทย/อังกฤษ, email, JWT, payment fields
- **Array expansion** — สร้าง list data จำนวนเท่าที่ต้องการ
- **Request validation** — ตรวจ type ของ body (`string`, `number`, `string[]` ฯลฯ)
- **Multi-config loading** — โหลดหลายไฟล์หรือทั้งโฟลเดอร์
- **Zero-cloud** — รัน local ได้เลย ไม่ส่ง data ออกนอกเครื่อง

---

## Requirements

- [Rust](https://rustup.rs/) 1.70+ (edition 2024)
- [Node.js](https://nodejs.org/) 18+ — สำหรับ compile VS Code extension (optional)

---

## Quick Start

```bash
# clone & เข้าโปรเจกต์
cd siammock

# รัน server (โหลดทุกไฟล์ใน mock/ โดย default)
cargo run

# หรือระบุ command ชัดเจน
cargo run -- start
```

Server จะ listen ที่ `http://localhost:4300`

```bash
# ทดสอบ
curl http://localhost:4300/api/v1/users
curl -X POST http://localhost:4300/api/payment \
  -H "Content-Type: application/json" \
  -d '{"amount": 1500, "currency": "THB"}'
```

---

## CLI

```bash
siammock start [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-c, --config` | `mock` | ไฟล์ JSON, โฟลเดอร์, หรือหลาย path คั่นด้วย comma |
| `-p, --port` | `4300` | Port ของ server |
| `--host` | `0.0.0.0` | Host ที่ bind |
| `--data` | `data` | โฟลเดอร์ CSV ที่ export จาก database |

### ตัวอย่าง

```bash
# โหลดทั้งโฟลเดอร์ mock/
cargo run -- start -c mock

# โหลดไฟล์เดียว
cargo run -- start -c mock/payment.json

# โหลดหลายไฟล์
cargo run -- start -c mock/default.json,mock/payment.json

# เปลี่ยน port
cargo run -- start -c mock --port 8080
```

### Hot reload (development)

```bash
cargo install cargo-watch

cargo watch -x 'run -- start -c mock'
```

---

## VS Code / Cursor Extension

SiamMock มี extension สำหรับ **VS Code** และ **Cursor** ช่วยเขียน config ได้เร็วขึ้น — มี autocomplete, JSON schema, realtime validation และไอคอนไฟล์เฉพาะ

Extension อยู่ในโฟลเดอร์ `editor/siammock/` (publisher: `siammock`, ชื่อ **SiamMock Config Editor**)

### สิ่งที่ extension ทำให้

| ฟีเจอร์ | รายละเอียด |
|---------|------------|
| **Realtime validation** | ตรวจ config ขณะพิมพ์ โดยเรียก binary `SiamMock validate` ใน background |
| **Autocomplete** | แนะนำ HTTP method, placeholder `{{...}}`, field keys, body types |
| **JSON Schema** | ตรวจโครงสร้าง JSON ตาม `siammock.schema.json` |
| **Snippets** | พิมพ์ `route` แล้วกด Tab เพื่อสร้าง route template |
| **ไอคอนไฟล์** | ไฟล์ `.jsonsi` แสดงไอคอน SiamMock ใน Explorer |

### Requirements (ก่อนติดตั้ง extension)

- [Node.js](https://nodejs.org/) 18+ (สำหรับ compile extension)
- Rust toolchain + build binary แล้ว (`cargo build`) — extension ใช้ binary นี้ validate config

### ติดตั้ง (ครั้งแรก)

```bash
# 1. build SiamMock binary (validator ใช้ตัวนี้)
cargo build

# 2. compile extension
cd editor/siammock
npm install
npm run compile
```

จากนั้นใน **VS Code** หรือ **Cursor**:

1. เปิด Command Palette (`Cmd+Shift+P` / `Ctrl+Shift+P`)
2. เลือก **Developer: Install Extension from Location...**
3. เลือกโฟลเดอร์ `editor/siammock`
4. รัน **Developer: Reload Window**

> โปรเจกต์แนะนำ extension นี้ใน `.vscode/extensions.json` — เมื่อเปิด workspace จะมี popup ให้ติดตั้งได้

### อัปเดต extension หลังแก้โค้ด

```bash
cd editor/siammock
npm run compile
```

แล้ว **Reload Window** ใน VS Code / Cursor

ถ้าจะแจกเป็นไฟล์ `.vsix`:

```bash
cd editor/siammock
npm install -g @vscode/vsce   # ครั้งแรกเท่านั้น
npm run package               # ได้ไฟล์ .vsix
```

ติดตั้งจาก `.vsix`: Command Palette → **Extensions: Install from VSIX...**

### ประเภทไฟล์ config

| ไฟล์ | คำอธิบาย |
|------|----------|
| `*.jsonsi` | ไฟล์ config หลักของ SiamMock (มี syntax + ไอคอนเฉพาะ) |
| `mock/*.json` | รูปแบบเดิม ยังรองรับ validation และ autocomplete |

แนะนำสร้าง config ใหม่เป็น `.jsonsi` เช่น `mock/users.jsonsi`

### วิธีใช้งาน

#### 1. Realtime validation

เปิดไฟล์ `.jsonsi` หรือ `mock/*.json` — extension จะ validate อัตโนมัติเมื่อพิมพ์ (debounce ~400ms) และเมื่อ save

- error/warning แสดงใน **Problems** panel
- ถ้า binary ยังไม่ build จะขึ้นข้อความให้รัน `cargo build`

Validate เองได้จาก Command Palette:

```
SiamMock: Validate Active File
```

หรือจาก terminal:

```bash
cargo run -- validate -c mock/default.json
cargo run -- validate -c mock/
```

#### 2. Autocomplete

| จุดที่พิมพ์ | สิ่งที่แนะนำ |
|-------------|-------------|
| `"method": "` | GET, POST, PUT, PATCH, DELETE, … |
| `{{` | uuid, timestamp, thai_name, email, param:id, body:field, csv:… |
| `{` หรือ `,` ใน route object | path, method, response, request, summary |
| `"body": { ... "` | string, number, boolean, string[], string (required), … |

> placeholder `{{param:id}}` และ `{{body:field}}` จะแนะนำจาก path/body ใน route เดียวกันอัตโนมัติ

#### 3. Snippets

ในไฟล์ `.jsonsi` หรือ `mock/*.json` พิมพ์ `route` แล้วกด **Tab** เพื่อแทรก route template พร้อมโครงสร้าง request/response

#### 4. รัน mock server คู่กับ extension

```bash
# terminal 1 — server
cargo run -- start -c mock

# terminal 2 (optional) — hot reload config
cargo watch -x 'run -- start -c mock'
```

แก้ config ใน editor → ดู validation ทันที → save → ทดสอบด้วย curl หรือ REST Client

### Extension settings

ตั้งค่าใน **Settings** (`Cmd+,` / `Ctrl+,`) ค้นหา `siammock`:

| Setting | Default | คำอธิบาย |
|---------|---------|----------|
| `siammock.binaryPath` | auto-detect | path ไปยัง SiamMock binary |
| `siammock.validateOnChange` | `true` | validate ขณะพิมพ์ |
| `siammock.validateDebounceMs` | `400` | หน่วง ms ก่อน re-validate |
| `siammock.mockJsonGlob` | `mock/*` | glob ของไฟล์ `.json` ที่ validate |

ตัวอย่างใน `.vscode/settings.json`:

```json
{
  "siammock.binaryPath": "${workspaceFolder}/target/debug/SiamMock",
  "siammock.validateOnChange": true,
  "siammock.validateDebounceMs": 400,
  "siammock.mockJsonGlob": "mock/*"
}
```

> บน macOS/Linux binary อยู่ที่ `target/debug/SiamMock` — บน Windows ใช้ `SiamMock.exe`

### Troubleshooting

| ปัญหา | วิธีแก้ |
|-------|---------|
| ไม่มี autocomplete | Reload Window หลังติดตั้ง extension |
| Validation ไม่ทำงาน | รัน `cargo build` ให้ binary อยู่ใน `target/debug/` |
| ไฟล์ `.json` ไม่ validate | ตรวจว่าไฟล์อยู่ใน glob `siammock.mockJsonGlob` (default: `mock/*`) |
| แก้ extension แล้วไม่เห็นผล | `npm run compile` แล้ว Reload Window |

รายละเอียดเพิ่มเติม: [`editor/siammock/README.md`](editor/siammock/README.md)

---

## Config Format

ไฟล์ config เป็น JSON ที่มี `routes` array:

```json
{
  "routes": [
    {
      "path": "/api/v1/users/:id",
      "method": "GET",
      "summary": "Get user by ID",
      "request": {
        "query_params": {
          "page": 1,
          "limit": 10
        },
        "headers": {
          "Authorization": "Bearer token"
        },
        "body": {
          "name": "string"
        }
      },
      "response": {
        "status": 200,
        "headers": {
          "X-Request-Id": "{{uuid}}"
        },
        "body": {
          "success": true,
          "data": {
            "id": "{{param:id}}",
            "name": "{{thai_name}}",
            "created_at": "{{timestamp}}"
          }
        }
      }
    }
  ]
}
```

### Route fields

| Field | Required | Description |
|-------|----------|-------------|
| `path` | ✅ | URL path — ใช้ `:param` สำหรับ path parameter เช่น `/users/:id` |
| `method` | ✅ | `GET`, `POST`, `PUT`, `PATCH`, `DELETE` |
| `summary` | — | คำอธิบาย route (documentation) |
| `request` | — | Spec ของ request (optional ทั้ง block) |
| `response` | ✅ | Status code + body ที่จะ return |

### Request spec

| Field | Description |
|-------|-------------|
| `headers` | ตัวอย่าง headers (documentation — ยังไม่ enforce) |
| `query_params` | ตัวอย่าง query params (documentation — ยังไม่ enforce) |
| `body` | Schema สำหรับ validate request body |

### Response spec

| Field | Description |
|-------|-------------|
| `status` | HTTP status code |
| `headers` | Response headers (documentation — ยังไม่ enforce) |
| `body` | JSON template ที่จะ render ก่อนส่งกลับ |

---

## Placeholders

ใส่ใน `response.body` (string value) แล้วระบบจะแทนที่ตอน runtime:

### ทั่วไป

| Placeholder | Output |
|-------------|--------|
| `{{uuid}}` | UUID v4 |
| `{{timestamp}}` | ISO 8601 datetime |
| `{{random_number}}` | ตัวเลขสุ่ม 1–10,000 |
| `{{random_string}}` | สตริง alphanumeric สุ่ม |
| `{{jwt_token}}` | JWT token (HS256, mock) |

### Mock data

| Placeholder | Output |
|-------------|--------|
| `{{thai_name}}` | ชื่อไทยสุ่ม |
| `{{en_name}}` | ชื่ออังกฤษสุ่ม |
| `{{email}}` | Email สุ่ม |
| `{{currency}}` | สกุลเงินสุ่ม (THB, USD, …) |
| `{{payment_method}}` | วิธีชำระเงินสุ่ม |
| `{{payment_status}}` | สถานะ payment สุ่ม |
| `{{status}}` | สถานะทั่วไปสุ่ม |

### จาก request

| Placeholder | Output |
|-------------|--------|
| `{{param:field}}` | ค่าจาก path param เช่น `/users/:id` → `{{param:id}}` |
| `{{body:field}}` | ค่าจาก request JSON body (POST/PUT/PATCH) |

### ใน repeat loop

| Placeholder | Output |
|-------------|--------|
| `{{index}}` | ลำดับ 0, 1, 2, … |
| `{{index:1}}` | ลำดับ 1, 2, 3, … |

### จาก CSV (export จาก Database)

วางไฟล์ `.csv` ในโฟลเดอร์ `data/` แล้วอ้าง column ใน JSON config:

| Placeholder | Output |
|-------------|--------|
| `{{csv:users.csv:email}}` | ค่าจาก column `email` (สุ่ม 1 แถว) |
| `{{csv:users.csv:first_name}}` | ค่าจาก column `first_name` |
| `{{csv_count:users.csv}}` | จำนวนแถวทั้งหมดในไฟล์ |

**ตัวอย่าง CSV** (`data/users.csv`):

```csv
id,first_name,last_name,email,phone,role,status
1,สมชาย,ใจดี,somchai@example.com,0812345678,customer,active
2,สมหญิง,รักเรียน,somying@example.com,0898765432,customer,active
```

**ใช้ใน response:**

```json
{
  "path": "/api/v1/users",
  "method": "GET",
  "response": {
    "status": 200,
    "body": {
      "success": true,
      "data": {
        "repeat": "{{csv_count:users.csv}}",
        "item": {
          "id": "{{csv:users.csv:id}}",
          "first_name": "{{csv:users.csv:first_name}}",
          "last_name": "{{csv:users.csv:last_name}}",
          "email": "{{csv:users.csv:email}}",
          "phone": "{{csv:users.csv:phone}}"
        }
      }
    }
  }
}
```

> เมื่ออยู่ใน `repeat` loop ระบบจะใช้แถวที่ตรงกับ `{{index}}` — column ใน item เดียวกันมาจากแถวเดียวกัน

**Export จาก Database:**

```sql
-- PostgreSQL
COPY (SELECT id, first_name, email FROM users) TO 'data/users.csv' CSV HEADER;

-- MySQL
SELECT id, first_name, email FROM users
INTO OUTFILE 'data/users.csv'
FIELDS TERMINATED BY ',' ENCLOSED BY '"'
LINES TERMINATED BY '\n';
```

---

## Dynamic Lists

### วิธีที่ 1 — `repeat` + `item` (แนะนำ)

```json
{
  "data": {
    "repeat": 20,
    "item": {
      "id": "{{uuid}}",
      "name": "{{thai_name}}",
      "no": "{{index:1}}"
    }
  },
  "total": 20
}
```

### วิธีที่ 2 — `total` + array template เดียว

```json
{
  "users": [
    { "id": "{{uuid}}", "name": "{{thai_name}}" }
  ],
  "total": 20
}
```

→ ขยาย `users` เป็น 20 รายการ (แต่ละรายการได้ uuid/name ใหม่)

### Nested arrays

`repeat` ซ้อน `repeat` ได้:

```json
{
  "orders": {
    "repeat": 3,
    "item": {
      "order_id": "{{uuid}}",
      "items": {
        "repeat": 5,
        "item": {
          "sku": "{{uuid}}",
          "qty": 1
        }
      }
    }
  }
}
```

> จำกัดสูงสุด 1,000 รายการต่อ `repeat`

---

## Request Validation

เมื่อ route มี `request.body` ระบบจะ validate incoming JSON:

```json
"request": {
  "body": {
    "amount": "number",
    "currency": "string",
    "tags": "string[]"
  }
}
```

### Supported types

| Type | ตรวจ |
|------|------|
| `string` | เป็น string |
| `number` | เป็น number |
| `boolean` | เป็น boolean |
| `array` | เป็น array |
| `object` | เป็น object |
| `string[]` | array ของ string ทุกตัว |
| `number[]` | array ของ number ทุกตัว |
| `boolean[]` | array ของ boolean ทุกตัว |
| `string (required)` | ต้องมี field + เป็น string |

ค่าที่เป็น **ตัวอย่าง** เช่น `"user@example.com"` จะถือเป็น documentation — **ไม่ validate**

Validation ไม่ผ่าน → HTTP `400`:

```json
{
  "errors": {
    "amount": "expected number, got string"
  }
}
```

---

## Project Structure

```
siammock/
├── mock/                  # JSON config files
│   ├── default.json
│   └── payment.json
├── data/                  # CSV exports from database
│   └── users.csv
├── editor/
│   └── siammock/          # VS Code / Cursor extension
├── src/
│   ├── main.rs
│   ├── app.rs             # Server startup
│   ├── cli.rs             # CLI (clap)
│   ├── config/
│   │   ├── schema.rs      # Config structs
│   │   └── loader.rs      # Load & merge configs
│   ├── handlers/
│   │   └── mock.rs        # Route dispatch handler
│   ├── response/
│   │   ├── template.rs    # Placeholder engine
│   │   └── constants.rs   # Mock data pools
│   ├── router/
│   │   └── builder.rs     # Axum router builder
│   └── validation/
│       └── body.rs        # Request body validation
└── Cargo.toml
```

---

## Logging

```bash
# เปิด debug log
RUST_LOG=debug cargo run -- start
```

---

## Roadmap

- [ ] Response headers enforcement
- [ ] Query params matching & placeholders
- [ ] Webhook dispatcher + latency simulator
- [ ] Thai platform templates (PromptPay, LINE OA, Omise)
- [ ] npm package (`npx siammock start`)

---

## License

MIT
