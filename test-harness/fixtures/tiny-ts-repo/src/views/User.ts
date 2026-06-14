// Used as a type by index.ts — alive
export interface UserProfile {
  id: string;
  name: string;
  email: string;
}

// PLANTED: structural duplicate of UserProfile (P4 shape duplication should catch this)
export interface UserData {
  id: string;
  name: string;
  email: string;
}

// PLANTED: dead type alias
export type Foo = { bar: number };

// PLANTED: any-contaminated shape (anyContamination annotation should fire when wired up)
export interface BadShape {
  payload: any;
  meta: any;
  raw: any;
}
