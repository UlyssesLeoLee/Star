// frontend/src/mocks/data/validation.ts
// Runtime mock endpoints do not fabricate Task Card validation history.
// The demo validation records once associated with wi-001..wi-004 were retired.
// Keep these runtime arrays empty so mock mode cannot display invented task history.
// Unit tests use isolated test-only records with test-prefixed IDs.

import type { ValidationResultRecord, AcceptanceCoverageReport } from "@/types/ids";

// Task Card data must come from the authorized Run backend; tests own their synthetic cases.
export const MOCK_VALIDATION_RESULTS: ReadonlyArray<ValidationResultRecord> = [];
export const MOCK_ACCEPTANCE_COVERAGE: ReadonlyArray<AcceptanceCoverageReport> = [];
