/*
@cypher schema=1 source_sha256=8862ffbca8550f444a361e6e802a608c36c380a24d010b5fbbf7017ae114710d
MERGE (self:File {path:"frontend/src/lib/gantt/dateUtils.ts"})
MERGE (utcMidnight:Function {name:"utcMidnight", language:"typescript", visibility:"private"})
MERGE (toUtcCalendarDate:Function {name:"toUtcCalendarDate", language:"typescript", visibility:"public"})
MERGE (addUtcCalendarDays:Function {name:"addUtcCalendarDays", language:"typescript", visibility:"public"})
MERGE (differenceInUtcCalendarDays:Function {name:"differenceInUtcCalendarDays", language:"typescript", visibility:"public"})
MERGE (self)-[:DEFINES]->(utcMidnight)
MERGE (self)-[:DEFINES]->(toUtcCalendarDate)
MERGE (self)-[:DEFINES]->(addUtcCalendarDays)
MERGE (self)-[:DEFINES]->(differenceInUtcCalendarDays)
MERGE (utcMidnight)-[:CALLS]->(toUtcCalendarDate)
MERGE (addUtcCalendarDays)-[:CALLS]->(utcMidnight)
MERGE (differenceInUtcCalendarDays)-[:CALLS]->(utcMidnight)
@endcypher
*/

const UTC_DAY_MS = 24 * 60 * 60 * 1000;

function utcMidnight(value: string): number {
  const calendarDate = toUtcCalendarDate(value);
  const timestamp = Date.parse(`${calendarDate}T00:00:00.000Z`);
  if (!Number.isFinite(timestamp)) {
    throw new RangeError(`Invalid UTC calendar date: ${value}`);
  }
  return timestamp;
}

/** Return the UTC calendar day for an ISO date or timestamp, independent of local timezone. */
export function toUtcCalendarDate(value: string): string {
  const timestamp = Date.parse(value);
  if (!Number.isFinite(timestamp)) {
    throw new RangeError(`Invalid ISO date: ${value}`);
  }
  return new Date(timestamp).toISOString().slice(0, 10);
}

/** Add whole calendar days using UTC arithmetic, without local DST or timezone conversion. */
export function addUtcCalendarDays(value: string, days: number): string {
  if (!Number.isSafeInteger(days)) {
    throw new RangeError(`UTC calendar day offset must be a safe integer: ${days}`);
  }
  const date = new Date(utcMidnight(value));
  date.setUTCDate(date.getUTCDate() + days);
  return date.toISOString().slice(0, 10);
}

/** Return later minus earlier in whole UTC calendar days. */
export function differenceInUtcCalendarDays(later: string, earlier: string): number {
  return (utcMidnight(later) - utcMidnight(earlier)) / UTC_DAY_MS;
}
