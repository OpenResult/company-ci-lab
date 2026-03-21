<<<<<<< ours
<<<<<<< ours
<<<<<<< ours
export function greet(name: string): string {
  return `hello, ${name}`;
}
=======
=======
>>>>>>> theirs
=======
>>>>>>> theirs
export interface ReleaseInfo {
  readonly component: string;
  readonly pipeline: 'verify' | 'package' | 'publish';
}

export function greet(name: string): string {
  return `hello, ${name}`;
}

export function formatReleaseTag(info: ReleaseInfo): string {
  return `${info.component}:${info.pipeline}`;
}
<<<<<<< ours
<<<<<<< ours
>>>>>>> theirs
=======
>>>>>>> theirs
=======
>>>>>>> theirs
