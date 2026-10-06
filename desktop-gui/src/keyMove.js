// [LINE BUDGET AUDIT] 14/250
// A key's ID is its physical slot; move the content, not the ID.
export function swapKeyContents(keys, sourceId, targetId) {
  const source = keys.find(key => key.id === sourceId)
  const target = keys.find(key => key.id === targetId)
  if (!source || !target || sourceId === targetId) return null
  const content = key => ({ label: key.label, action: key.action, icon: key.icon })
  return keys.map(key => {
    if (key.id === sourceId) return { ...key, ...content(target) }
    if (key.id === targetId) return { ...key, ...content(source) }
    return key
  })
}
