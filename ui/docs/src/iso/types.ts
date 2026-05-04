export type ZLayerName =
  | 'Document'
  | 'Pipeline'
  | 'Constraint'
  | 'Legal'
  | 'FormalProof'
  | 'Attestation';

export type SemanticTypeName =
  | 'document'
  | 'pipeline'
  | 'constraint'
  | 'gate'
  | 'legal'
  | 'solver'
  | 'result'
  | 'flag'
  | 'issue'
  | 'proof'
  | 'attestation'
  | 'unknown';

export interface VisualizationSpec {
  semantic_type: SemanticTypeName;
  z_layer: ZLayerName;
  rhai_dsl: string;
  description: string;
}

export interface VizManifestEntry {
  type_name: string;
  spec: VisualizationSpec;
}

export interface VizManifest {
  version: string;
  objects: VizManifestEntry[];
}

export const LAYER_COLORS: Record<ZLayerName, string> = {
  Document:    '#334155',
  Pipeline:    '#1d4ed8',
  Constraint:  '#7c3aed',
  Legal:       '#b91c1c',
  FormalProof: '#0f766e',
  Attestation: '#b45309',
};

export const LAYER_BASE_Z: Record<ZLayerName, number> = {
  Document:    0,
  Pipeline:    136,
  Constraint:  272,
  Legal:       408,
  FormalProof: 544,
  Attestation: 680,
};
