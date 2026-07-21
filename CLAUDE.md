# Project Context: SiamMock

## Overview
SiamMock is a high-speed Mock API and Webhook Simulator designed to eliminate development bottlenecks. It allows frontend and backend teams to work concurrently without waiting for actual production servers. The project utilizes a hybrid architecture combining Rust (as the high-performance core processing engine) and TypeScript (for user integration, configuration management, and the CLI tool).

## Tech Stack
- Backend/Core Engine: Rust (Axum, Tokio, serde_json, rand, uuid)
- Integration/CLI: TypeScript (Node.js, NAPI-RS)

## Detailed Feature Scope

### 1. Custom JSON Verification & Request Matching
- The system must accept incoming HTTP POST requests with dynamic, unstructured JSON bodies parsed into `serde_json::Value`.
- It performs a validation loop against a user-defined expected schema stored in memory or cache.
- The engine checks for required keys and validates data types (e.g., String, Number, Boolean, Arrays, Objects).
- If any required key is missing or data types mismatch, the system must immediately reject the request with an HTTP 400 Bad Request status, returning a precise error payload mapping the exact invalid fields.

### 2. Intelligent Dynamic Response Generation
- The core engine evaluates a JSON response template previously configured by the developer.
- It scans the template and replaces dynamic placeholder tags, specifically:
  - `{{uuid}}`: Replaced with a newly generated v4 UUID.
  - `{{random_number}}`: Replaced with a cryptographically secure random number or range-based integer.
  - `{{timestamp}}`: Replaced with the current ISO or epoch timestamp.
- The text processing must happen via optimized string manipulation techniques in Rust before parsing back into JSON, ensuring responses are delivered in under 1 millisecond (< 1ms).

### 3. Advanced Webhook Dispatcher & Latency Simulator
- Latency Simulator: Simulates realistic network degradation or slow third-party dependencies by pausing the response cycle using non-blocking asynchronous timers via `tokio::time::sleep`.
- Async Webhook Dispatcher: Emits outbound webhook events (simulating services like PromptPay, LINE OA, or Omise) to a client's server. The execution must be offloaded instantly to a background task using `tokio::spawn` to prevent blocking the main HTTP listener thread, allowing the mock server to return an immediate HTTP 200 OK to the sender.
- Retry Queue: Includes a background task manager that catches failed webhook deliveries (e.g., HTTP 5xx responses or network timeouts) and enqueues them for automatic retries using an Exponential Backoff strategy.

### 4. Zero-Cloud Local CLI
- The engine must be compiled into a single native binary using NAPI-RS and packed into an npm package.
- This allows developers to run `npx siammock start` from their local terminal, spawning a local mock server instance on localhost (127.0.0.1) that operates entirely offline without sending any project data to external clouds.

## Strategic Core Focus & Unfair Advantages
- Rustification Speed: The system must minimize memory allocations and maximize CPU throughput to easily sustain load/stress testing benchmarks exceeding 1,000+ Requests/Sec on minimal server resources.
- Thai Local Templates: Built-in pre-configured mock templates for major Thai local developer platforms (e.g., exact JSON payload signatures for PromptPay notification webhooks, LINE Official Account messaging events, and Omise payment gateways).
- Enterprise Data Privacy: Designed from the ground up to respect data compliance policies of local tech firms and financial institutes by ensuring all validation logic can function in a zero-cloud local environment.

---
Instructions for AI: Please generate the initial project directory structure, set up the Cargo.toml file with necessary dependencies, and begin writing the implementation for Feature 1 (Custom JSON Verification) using the Axum framework in Rust.