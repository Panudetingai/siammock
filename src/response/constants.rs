pub const CURRENCIES: &[&str] = &[
    "THB", "USD", "EUR", "JPY", "KRW", "CNY", "INR", "BRL", "MXN", "ZAR",
];

pub const PAYMENT_METHODS: &[&str] = &[
    "credit_card",
    "debit_card",
    "bank_transfer",
    "mobile_wallet",
    "e_wallet",
    "crypto_currency",
    "other",
];

pub const PAYMENT_STATUSES: &[&str] = &[
    "pending",
    "completed",
    "failed",
    "refunded",
    "cancelled",
    "expired",
    "partially_refunded",
    "partially_completed",
    "partially_failed",
    "partially_refunded",
];

pub const STATUS: &[&str] = &[
    "success",
    "error",
    "pending",
    "completed",
    "failed",
    "refunded",
    "cancelled",
    "expired",
];
