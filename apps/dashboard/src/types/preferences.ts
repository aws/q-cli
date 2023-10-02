export type PrefDefault = boolean | string | number | null | never[]

export type Pref = {
  id: string,
  title: string,
  description?: string,
  example?: string,
  type: string,
  inverted?: boolean
  default: PrefDefault
  popular?: boolean
  options?: string[]
}