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
