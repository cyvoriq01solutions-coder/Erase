export const SETUP_ARTIFACT = "setup" as const;

export const RELEASE_OBJECTS = {
  setup: {
    key: "releases/0.3.0/CYVRA-Erase-0.3.0-x64-setup.exe",
    filename: "CYVRA-Erase-0.3.0-x64-setup.exe",
    contentType: "application/octet-stream",
  },
} as const;

export type ReleaseArtifact = keyof typeof RELEASE_OBJECTS;

export function resolveReleaseArtifact(
  name: string,
): (typeof RELEASE_OBJECTS)[ReleaseArtifact] | null {
  if (name !== SETUP_ARTIFACT) {
    return null;
  }
  return RELEASE_OBJECTS.setup;
}

export interface ReleaseStore {
  head(key: string): Promise<{ size: number } | null>;
  get(key: string): Promise<{
    body: ReadableStream;
    size: number;
    httpEtag?: string;
  } | null>;
}

export async function isSetupPackagePresent(
  store: ReleaseStore,
): Promise<boolean> {
  const object = await store.head(RELEASE_OBJECTS.setup.key);
  return object !== null && object.size > 0;
}
