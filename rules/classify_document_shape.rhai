// rules/classify_document_shape.rhai
//
// Classification rule: Document Shape Detection
//
// Classifies the SHAPE of an ingested document — not its individual
// transactions — to route it to the correct extraction profile in the
// pipeline.
//
// Entry point called by the Rust engine:
//
//   fn classify_document(doc)
//
// Input map:
//   doc.filename        — e.g. "WF--BH-CHK--2024-01--statement.csv"
//   doc.doc_type        — DocType slug: "SpreadsheetCsv", "Pdf", "SpreadsheetXlsx"
//   doc.sample_content  — first ~2KB of file content (string)
//   doc.file_size_bytes — file size as a decimal string
//
// Output map:
//   vendor            — StatementVendor slug, e.g. "WellsFargo"
//   account_type      — "checking" | "savings" | "brokerage" | "crypto"
//   statement_format  — "csv_ofx" | "pdf_tabular" | "xlsx_native" | "csv_generic"
//   currency          — "USD" | "AUD" | "EUR" | "GBP"
//   confidence        — float 0.0–1.0
//   signals           — array of matched signal names
//   reason            — human-readable explanation for the audit trail
//
// Filename convention (VENDOR--ACCOUNT--YYYY-MM--DOCTYPE):
//   The source filename MUST follow this naming convention.
//   The vendor slug embedded in the filename is the primary routing signal.
//   See: AGENTS.md § validate_source_filename.
//
// See: document_shape.rs — DocumentShape, StatementVendor, classify_document_shape()

fn classify_document(doc) {
    // --- Guard: handle missing/empty doc ---
    if doc == () {
        return #{
            vendor:           "Unknown",
            account_type:     "checking",
            statement_format: "csv_generic",
            currency:         "USD",
            confidence:       0.0,
            signals:          [],
            reason:           "doc was null/empty"
        };
    }

    // --- Extract fields with guards ---
    let filename = "";
    let doc_type = "";
    let sample   = "";

    if doc.contains("filename") {
        filename = doc["filename"];
    }
    if doc.contains("doc_type") {
        doc_type = doc["doc_type"];
    }
    if doc.contains("sample_content") {
        sample = doc["sample_content"];
    }

    // Handle empty-after-extraction case
    if filename == "" && doc_type == "" && sample == "" {
        return #{
            vendor:           "Unknown",
            account_type:     "checking",
            statement_format: "csv_generic",
            currency:         "USD",
            confidence:       0.0,
            signals:          [],
            reason:           "all doc fields empty — cannot classify"
        };
    }

    let filename_lower = filename.to_lower();
    let sample_lower   = sample.to_lower();

    // =========================================================
    // VENDOR DETECTION
    // Primary signal: vendor slug embedded in filename
    // (VENDOR--ACCOUNT--YYYY-MM--DOCTYPE convention)
    // =========================================================

    let vendor        = "Unknown";
    let vendor_signal = "";

    if filename_lower.contains("wellsfargo") || filename_lower.contains("--wf--") || filename_lower.starts_with("wf--") {
        vendor        = "WellsFargo";
        vendor_signal = "filename_vendor_slug";
    } else if filename_lower.contains("chase") || filename_lower.contains("jpmc") {
        vendor        = "Chase";
        vendor_signal = "filename_vendor_slug";
    } else if filename_lower.contains("bankofamerica") || filename_lower.contains("--boa--") || filename_lower.starts_with("boa--") {
        vendor        = "BankOfAmerica";
        vendor_signal = "filename_vendor_slug";
    } else if filename_lower.contains("hsbc") {
        vendor        = "Hsbc";
        vendor_signal = "filename_vendor_slug";
    } else if filename_lower.contains("--anz--") || filename_lower.starts_with("anz--") {
        vendor        = "Anz";
        vendor_signal = "filename_vendor_slug";
    } else if filename_lower.contains("commbank") || filename_lower.contains("--cba--") || filename_lower.starts_with("cba--") {
        vendor        = "Commbank";
        vendor_signal = "filename_vendor_slug";
    } else if filename_lower.contains("westpac") {
        vendor        = "WestpacAu";
        vendor_signal = "filename_vendor_slug";
    } else if filename_lower.contains("ibkr") || filename_lower.contains("interactive") {
        vendor        = "Interactive";
        vendor_signal = "filename_vendor_slug";
    } else if filename_lower.contains("coinbase") {
        vendor        = "Coinbase";
        vendor_signal = "filename_vendor_slug";
    } else if filename_lower.contains("kraken") {
        vendor        = "Kraken";
        vendor_signal = "filename_vendor_slug";
    }

    // Secondary signal: sample content institution hints
    // (lower priority than filename; used to boost confidence)
    let content_vendor_signal = "";
    if vendor == "Unknown" {
        if sample_lower.contains("wells fargo") || sample_lower.contains("wellsfargo") {
            vendor               = "WellsFargo";
            content_vendor_signal = "sample_content_institution";
        } else if sample_lower.contains("jpmorgan chase") || sample_lower.contains("chase bank") {
            vendor               = "Chase";
            content_vendor_signal = "sample_content_institution";
        } else if sample_lower.contains("bank of america") {
            vendor               = "BankOfAmerica";
            content_vendor_signal = "sample_content_institution";
        } else if sample_lower.contains("hsbc") {
            vendor               = "Hsbc";
            content_vendor_signal = "sample_content_institution";
        } else if sample_lower.contains("anz bank") || sample_lower.contains("australia and new zealand banking") {
            vendor               = "Anz";
            content_vendor_signal = "sample_content_institution";
        } else if sample_lower.contains("commonwealth bank") || sample_lower.contains("commbank") {
            vendor               = "Commbank";
            content_vendor_signal = "sample_content_institution";
        } else if sample_lower.contains("westpac") {
            vendor               = "WestpacAu";
            content_vendor_signal = "sample_content_institution";
        } else if sample_lower.contains("interactive brokers") || sample_lower.contains("ibkr") {
            vendor               = "Interactive";
            content_vendor_signal = "sample_content_institution";
        } else if sample_lower.contains("coinbase") {
            vendor               = "Coinbase";
            content_vendor_signal = "sample_content_institution";
        } else if sample_lower.contains("kraken") {
            vendor               = "Kraken";
            content_vendor_signal = "sample_content_institution";
        }
    }

    // =========================================================
    // ACCOUNT TYPE DETECTION
    // =========================================================

    let account_type = "checking";  // default

    if filename_lower.contains("brokerage") || filename_lower.contains("ibkr") || filename_lower.contains("interactive") || filename_lower.contains("invest") {
        account_type = "brokerage";
    } else if filename_lower.contains("savings") || filename_lower.contains("sav") {
        account_type = "savings";
    } else if filename_lower.contains("crypto") || filename_lower.contains("coinbase") || filename_lower.contains("kraken") {
        account_type = "crypto";
    } else if sample_lower.contains("brokerage account") || sample_lower.contains("investment account") {
        account_type = "brokerage";
    } else if sample_lower.contains("savings account") {
        account_type = "savings";
    } else if sample_lower.contains("bitcoin") || sample_lower.contains("ethereum") || sample_lower.contains("crypto") {
        account_type = "crypto";
    }

    // =========================================================
    // STATEMENT FORMAT DETECTION
    // =========================================================

    let statement_format       = "csv_generic";
    let format_signal          = "";

    if doc_type == "SpreadsheetCsv" {
        // OFX-style CSV: has FITID and TRNTYPE header columns
        if sample.contains("FITID") && sample.contains("TRNTYPE") {
            statement_format = "csv_ofx";
            format_signal    = "ofx_header_match";
        } else {
            statement_format = "csv_generic";
            format_signal    = "csv_header_match";
        }
    } else if doc_type == "Pdf" {
        statement_format = "pdf_tabular";
        format_signal    = "doc_type_pdf";
    } else if doc_type == "SpreadsheetXlsx" {
        statement_format = "xlsx_native";
        format_signal    = "doc_type_xlsx";
    }

    // =========================================================
    // CURRENCY DETECTION
    // =========================================================

    let currency = "USD";  // default

    // AUD signals take precedence for Australian institution filenames
    if filename.contains("AU") || sample.contains("AUD") || sample.contains("A$") {
        currency = "AUD";
    } else if sample.contains("EUR") || sample.contains("€") {
        currency = "EUR";
    } else if sample.contains("GBP") || sample.contains("£") {
        currency = "GBP";
    }

    // Australian vendors default to AUD if currency not otherwise detected
    if currency == "USD" {
        if vendor == "Anz" || vendor == "Commbank" || vendor == "WestpacAu" {
            currency = "AUD";
        }
    }

    // =========================================================
    // CONFIDENCE SCORING
    // =========================================================

    // Base confidence on signal quality:
    //   filename vendor slug:    +0.50 (primary routing signal per naming convention)
    //   content vendor signal:   +0.25 (secondary, less reliable)
    //   format signal matched:   +0.15
    //   format via doc_type:     +0.10
    //   no vendor detected:      confidence stays low

    let confidence = 0.0;
    let signals    = [];

    if vendor_signal != "" {
        confidence = confidence + 0.50;
        signals.push(vendor_signal);
    }
    if content_vendor_signal != "" {
        confidence = confidence + 0.25;
        signals.push(content_vendor_signal);
    }
    if format_signal != "" {
        if format_signal == "ofx_header_match" || format_signal == "csv_header_match" {
            confidence = confidence + 0.15;
        } else {
            confidence = confidence + 0.10;
        }
        signals.push(format_signal);
    }

    // Cap at 0.97 — never claim perfect certainty from heuristics alone
    if confidence > 0.97 {
        confidence = 0.97;
    }

    // =========================================================
    // REASON STRING (for audit trail)
    // =========================================================

    let reason = "";

    if vendor == "Unknown" {
        reason = "no vendor signals matched filename or sample content — classified as Unknown";
    } else if vendor_signal != "" && content_vendor_signal != "" {
        reason = "filename vendor slug and sample content both matched " + vendor;
    } else if vendor_signal != "" {
        reason = "filename vendor slug matched " + vendor + " per VENDOR--ACCOUNT--YYYY-MM--DOCTYPE convention";
    } else {
        reason = "sample content institution name matched " + vendor;
    }

    if format_signal != "" {
        reason = reason + "; format detected via " + format_signal;
    }

    #{
        vendor:           vendor,
        account_type:     account_type,
        statement_format: statement_format,
        currency:         currency,
        confidence:       confidence,
        signals:          signals,
        reason:           reason
    }
}
