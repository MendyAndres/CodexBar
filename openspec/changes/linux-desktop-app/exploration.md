# Exploración correctiva: `linux-desktop-app`

## Veredicto

**Rust** es la opción recomendada para el host Linux futuro: `gtk4-rs` + `libadwaita` y un adapter SNI/D-Bus propio. **Swift/Foundation + shim C/GDBus** solo conviene para un prototipo local, no como base de publicación. **Go no conviene**: `gotk3` es GTK3; `gotk4` declara cobertura incompleta y posibles fugas/crashes; Fyne no es GTK/libadwaita ni resuelve SNI.

La decisión debe separar prototipo experimental de producto open source. Los documentos actuales mezclan ambos objetivos y tratan como cerrados contratos de SNI, configuración, polling y descubrimiento de CLI que aún no están definidos.

## Estado actual con evidencia local

`CodexBarCore` y `CodexBarCLI` ya son la frontera Linux reutilizable. `CodexBarCLI serve` tiene cache TTL, stale fallback, coordinación por proveedor, autenticación para rutas no-loopback y dashboard redacted. La UI gráfica sigue siendo macOS-only. `Package.swift` declara `.macOS(.v14)` a nivel de paquete, por lo que no basta con agregar un producto Linux condicionado: hay que resolver la frontera de plataforma del manifest.

El payload actual (`Sources/CodexBarCLI/DashboardPayloads.swift`) es `Encodable` y vive en CLI; no existe todavía un DTO público compartido con decoder Linux. El router actual reconoce `/dashboard/v1/snapshot`, pero no tiene semántica provider-scoped `provider`/`refresh`. Por tanto, la spec de refresh por proveedor **sí exige cambios del backend**, no es solo una decisión de UI.

CodeGraph fue intentado primero y respondió `Not connected`; se hizo inspección dirigida. No se ejecutaron tests, instalaciones, builds, probes, Keychain ni cookies.

## Comparación C/Swift vs Go vs Rust

| Opción | GTK/libadwaita | Tray/SNI | Mantenimiento/distribución | Concurrencia/testabilidad | Carga para este repo |
|---|---|---|---|---|---|
| **C/Swift + GDBus shim** | Binding Swift de terceros incierto; C puede consumir GTK4/GIO del sistema. | Control máximo de `org.kde.StatusNotifierItem` y `com.canonical.dbusmenu`; hay que implementar ownership, menú y señales. GNOME requiere extensión AppIndicator/KStatusNotifierItem. | Binario Swift + runtime/deps del sistema + shim propio; publicación y ABI costosos. | Actors/Foundation son familiares; cruce actor/GLib/FFI exige disciplina. Fake D-Bus permite tests, pero FFI aumenta superficie. | Menor reescritura conceptual, mayor deuda Linux específica. |
| **Go** | `gotk3` solo GTK3. `gotk4` es generador GTK4 con APIs faltantes y advertencias explícitas de fugas/crashes. No hay una opción libadwaita equivalente claramente madura. | Fyne es toolkit propio, no GTK. `systray`/similares no constituyen una implementación SNI/AppIndicator GTK4 mantenida; habría que escribir D-Bus. | Binario fácil de distribuir, pero cgo/GTK siguen requiriendo runtime; Fyne rompe la dirección GNOME. | Goroutines simples para IPC; callbacks GObject/cgo y ownership añaden riesgo. | Baja alineación con Swift y duplica modelos/DTO. Solo tendría sentido como proyecto Go separado. |
| **Rust** | `gtk4-rs` es el binding GNOME con GTK4, GLib/GIO, documentación y MSRV publicada; `libadwaita` pertenece al ecosistema GNOME. | `gio` o `zbus` sirven para D-Bus; SNI/DBusMenu siguen siendo protocolo propio. Ayatana/AppIndicator es GTK3/C y no debe mezclarse con GTK4. | Mejor historia de binario reproducible y publicación open source; requiere runtime o Flatpak/deb. | Ownership/Send/Sync reducen errores; async + GLib requiere adaptación explícita. Tests puros y fake transport son sólidos. | Mayor reescritura, menor deuda futura. |

## Evidencia externa (consultada 2026-07-26)

- `gtk4-rs`: bindings GTK4, crates GLib/GIO y MSRV Rust 1.83: https://github.com/gtk-rs/gtk4-rs (confianza alta).
- `gotk3`: README declara GTK3: https://github.com/gotk3/gotk3 (confianza alta).
- `gotk4`: generador GTK4; README advierte cobertura incompleta, fugas y crashes: https://github.com/diamondburned/gotk4 (confianza alta).
- Fyne: toolkit Go propio inspirado en Material, con C compiler; no es GTK: https://github.com/fyne-io/fyne (confianza alta).
- SNI: especificación/repositorio de referencia: https://github.com/StatusNotifierItem/StatusNotifierItem (confianza media; endpoint histórico de freedesktop devolvió 404).
- El binding Swift `makoni/swift-adwaita` no fue validado en CI: mantener su estado como confianza baja hasta verificar versión, licencia, GTK4/libadwaita y Swift 6.2.

## Decisiones operativas propuestas

1. **SNI:** implementar `org.kde.StatusNotifierItem` + `com.canonical.dbusmenu` directamente. Detectar session bus y watcher; documentar que GNOME requiere extensión AppIndicator/KStatusNotifierItem. Ausencia = `unsupported`, nunca “healthy”.
2. **Configuración:** ventana de preferencias sobre un store/API tipado; edición directa TOML solo como enlace documentado. Validar antes de escribir, temporal en el mismo directorio, rename atómico, permisos `0600`, rollback y cero secretos en logs. CLI y GUI deben usar la misma operación.
3. **Polling:** separar TTL del backend del intervalo del tray. Definir `generatedAt/updatedAt`, stale thresholds, refresh manual coalescido por proveedor, backoff exponencial con jitter y límite de reintentos.
4. **CLI discovery:** override explícito, sibling del binario, rutas XDG conocidas y PATH; mostrar rutas intentadas y acción manual. Nunca instalar o descargar silenciosamente.
5. **Proceso/puerto:** usar `--port 0`; readiness debe devolver puerto, schema, versión e instance-id autenticado. Reservar listener antes de anunciar ready. No adoptar ni matar procesos ajenos; conflicto = `collision`. Solo PID propio recibe TERM, timeout, KILL y wait/reap; limitar crash loop.
6. **Adapter protocol Rust:**

```text
trait TrayAdapter {
  fn start(&mut self, initial: AppState) -> Result<(), TrayError>;
  fn apply(&mut self, state: AppState);
  fn events(&self) -> Receiver<TrayEvent>; // Refresh(id), OpenLink(url), Quit
  fn stop(&mut self);
}
```

El controller posee `Starting -> BackendUnavailable|Ready -> Degraded -> Stopping -> Stopped`. `AppState` incluye capability `supported|unsupported`, backend `starting|healthy|collision|crashed` y estado provider-scoped con `lastGood`, freshness, retry y error. El adapter no ejecuta providers ni decide ownership.

## Contradicciones y gaps

- Proposal dice “GTK4 + StatusNotifierItem/AppIndicator”; design evita correctamente AppIndicator GTK3, pero no fija fallback ni requisito de extensión. Debe decir SNI/DBusMenu propio; la extensión GNOME es requisito de entorno.
- Spec exige refresh provider-scoped; design propone una ruta que el CLI actual no soporta. Mantenerlo requiere modificar capability/backend y publicar DTO compartido; de lo contrario reducir v1 a snapshot completo.
- Proposal promete configuración; spec exige guardarla; design promete allowlist/atomicidad, pero no enumera campos seguros ni política de API keys.
- Spec prohíbe adopción de listener ajeno; design usa `port 0` sin readiness protocol para obtener el puerto. `/health` actual solo devuelve status/version.
- Backend ya tiene TTL/stale cache, pero los documentos no definen polling del frontend, retry, jitter ni carga.
- `.deb` aparece como posterior y soporte público como fuera de alcance; separar explícitamente spike local de release open source.

## Cambios exactos recomendados

- **proposal.md:** elegir Rust para publicación y Swift/C solo spike; declarar SNI/DBusMenu y extensión GNOME; marcar el contrato dashboard/provider-refresh como capability modificada; separar entregables local/publicación.
- **spec.md:** agregar requisitos verificables para discovery, readiness/instance ownership, atomic config, polling/backoff, estados `unsupported/degraded/collision`, lifecycle y eventos del adapter. Si se conserva refresh provider-scoped, especificar el cambio CLI; si no, quitar el MUST.
- **design.md:** reemplazar la decisión del binding Swift por Rust; eliminar AppIndicator como implementación GTK4; definir state machine, trait/event model, SNI object contract, readiness y resolución de CLI; corregir `Package.swift` y separar frontend Rust del core Swift.
- **No implementar todavía:** primero decidir prototipo Swift/C vs host Rust de publicación y refresh provider-scoped vs snapshot completo v1.

## Riesgos

- Tray invisible en GNOME aunque el binario compile.
- Deuda de FFI si se publica el prototipo Swift/C.
- Contrato HTTP incompleto que mezcle cambios de backend con UI.
- Fugas de secretos durante configuración, logging o argumentos del proceso.
- Runtime GTK/libadwaita y packaging no cubiertos por CI actual.

## Result Contract

- **Veredicto recomendado:** Rust para publicación; Swift/C para spike local; Go descartado.
- **Evidencia:** código local dirigido y fuentes externas citadas; CodeGraph no disponible; sin ejecución de validaciones prohibidas.
- **Skill resolution:** `sdd-explore` aplicado; `cognitive-doc-design` aplicado; `codexbar` leído sin ejecutar comandos; persistencia híbrida Engram/OpenSpec; artefacto en español por pedido explícito.
- **Ready for Proposal:** sí, después de esas dos decisiones de usuario; no para implementación inmediata.
