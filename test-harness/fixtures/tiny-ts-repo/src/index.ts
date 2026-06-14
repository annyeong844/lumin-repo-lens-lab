import { formatDate } from './utils/date.js';
import type { UserProfile } from './views/User.js';

export function main(user: UserProfile): string {
  return formatDate(new Date()) + ': ' + user.name;
}
