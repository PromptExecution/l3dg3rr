use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ── Contacts ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XeroContact {
    pub contact_i_d: String,
    pub name: String,
    #[serde(default)]
    pub email_address: Option<String>,
    #[serde(default)]
    pub is_supplier: bool,
    #[serde(default)]
    pub is_customer: bool,
    #[serde(default)]
    pub tax_number: Option<String>,
    #[serde(default)]
    pub account_number: Option<String>,
    #[serde(default)]
    pub contact_status: Option<String>,
}

impl XeroContact {
    pub fn id(&self) -> &str {
        &self.contact_i_d
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct ContactsResponse {
    pub contacts: Vec<XeroContact>,
}

// ── Chart of Accounts ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XeroAccount {
    pub account_i_d: String,
    pub code: Option<String>,
    pub name: String,
    #[serde(rename = "Type")]
    pub account_type: String,
    pub description: Option<String>,
    #[serde(default)]
    pub currency_code: Option<String>,
    pub status: Option<String>,
}

impl XeroAccount {
    pub fn id(&self) -> &str {
        &self.account_i_d
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct AccountsResponse {
    pub accounts: Vec<XeroAccount>,
}

// ── Bank Accounts ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XeroBankAccount {
    pub account_i_d: String,
    pub name: String,
    pub bank_account_number: Option<String>,
    pub bank_account_type: Option<String>,
    pub currency_code: String,
    pub status: Option<String>,
}

impl XeroBankAccount {
    pub fn id(&self) -> &str {
        &self.account_i_d
    }
}

// ── Invoices ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XeroInvoice {
    pub invoice_i_d: String,
    #[serde(default)]
    pub invoice_number: Option<String>,
    pub contact: XeroContactRef,
    pub date: Option<String>,
    pub due_date: Option<String>,
    #[serde(default)]
    pub amount_due: Option<Decimal>,
    #[serde(default)]
    pub amount_paid: Option<Decimal>,
    pub status: String,
    #[serde(rename = "Type")]
    pub invoice_type: String,
}

impl XeroInvoice {
    pub fn id(&self) -> &str {
        &self.invoice_i_d
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XeroContactRef {
    pub contact_i_d: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct InvoicesResponse {
    pub invoices: Vec<XeroInvoice>,
}

// ── Organization / Tenant ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XeroTenant {
    pub tenant_id: String,
    pub tenant_name: String,
    pub tenant_type: String,
}

/// Compact representation for agent display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XeroEntityRef {
    pub entity_type: String,
    pub xero_id: String,
    pub display_name: String,
}

impl From<&XeroContact> for XeroEntityRef {
    fn from(c: &XeroContact) -> Self {
        Self {
            entity_type: "contact".into(),
            xero_id: c.contact_i_d.clone(),
            display_name: c.name.clone(),
        }
    }
}

impl From<&XeroBankAccount> for XeroEntityRef {
    fn from(b: &XeroBankAccount) -> Self {
        Self {
            entity_type: "bank_account".into(),
            xero_id: b.account_i_d.clone(),
            display_name: b.name.clone(),
        }
    }
}

impl From<&XeroAccount> for XeroEntityRef {
    fn from(a: &XeroAccount) -> Self {
        Self {
            entity_type: "account".into(),
            xero_id: a.account_i_d.clone(),
            display_name: a.name.clone(),
        }
    }
}
