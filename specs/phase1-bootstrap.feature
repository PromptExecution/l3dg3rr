Feature: Phase 1 Contracts and Session Bootstrap

  Scenario: Reject malformed filename before mutation
    Given a source filename "WF-BH-CHK-2023-01-statement.pdf"
    When preflight validation runs
    Then validation fails
    And no ledger mutation is attempted

  Scenario: Parse manifest and list accounts for MCP session bootstrap
    Given a manifest with accounts WF-BH-CHK and CB-BTC
    When the turbo MCP list_accounts tool is called
    Then both account IDs are returned
    And workbook loading is not required

  Scenario: Initialize workbook contract
    Given an empty destination path tax-ledger.xlsx
    When workbook initialization runs
    Then all required sheet names exist
    And downstream phases can target the contract by name
