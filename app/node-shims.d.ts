declare module "node:fs" {
    export function readFileSync(path: string): { toString(encoding?: string): string };
    export function existsSync(path: string): boolean;
}

declare module "node:path" {
    export function resolve(...paths: string[]): string;
    export function dirname(path: string): string;
}

declare const process: {
    env: Record<string, string | undefined>;
};
