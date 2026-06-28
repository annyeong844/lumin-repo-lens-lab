# Vibe Coder UX Review — 2026-04-28

## 배경

`lumin-repo-lens-lab` skill의 7개 슬래시 명령어를 vibe coder 렌즈로 같이 점검한 결과 정리. 같은 세션 안에서 실제로 명령어를 돌리고 출력을 비평해 모은 패턴과, 다음 작업자가 받아 손댈 구체적 위치를 적었다.

이 문서는 **결정 + 발견 + 수정 타겟** 3축으로 구성되어 있다.

---

## 결정 — 방향성

이 도구의 1차 사용자는 **vibe coder**로 결정.

- 이유: vibe coder는 도구 없이 구조 점검 못 함 (도구 의존도 높음). maintainer는 도구 없이도 `.audit/*.json` 직접 읽을 수 있음 (도구 의존도 낮음).
- 따라서 *"vibe coder에게 따뜻하고, maintainer가 들어와도 거슬리지 않는 정도"* 가 두 페르소나 다 잡으려는 시도보다 자연스러움.

이 결정은 SKILL.md에 이미 의도로 적혀 있음:

> *"For vibe-coding users, keep the surface kind and short: ... keep raw JSON, FP ids, and canonical jargon in reserve unless proof is requested."*

→ 의도는 있으나 실행이 못 따라가고 있음. 이 문서는 그 격차를 좁히는 작업의 출발점.

---

## 점검 대상과 점수표

| 명령어 | 친화도 | 핵심 문제 |
|---|---|---|
| `:welcome` | ⭐⭐⭐ | 표현만 다듬으면 됨 (세션 중 일부 정리) |
| `:audit` (default) | ⭐⭐ | JSON 경로 / FP id가 본문에 새어나옴 |
| `:pre-write` | ⭐⭐ | 의도가 비면 침묵 (폴백 없음) |
| `:post-write` | ⭐ 위험 | 5개 lane 중 1개만 검사 → 거짓 "all clean" |
| `:check-canon` | ⭐ 위험 | drift 45건을 chat에 한 줄도 안 띄움 → 거짓 안심 |
| `:canon-draft` | ⭐ 위험 | draft 만들고도 chat에 한 줄도 안 띄움 → 영영 못 찾음 |
| `:refactor-plan` | ⭐⭐ | 번역 표 있지만 강제 안 됨 + 템플릿 예시가 maintainer 톤 |

---

## 메타-패턴 3가지 (시급한 순)

### A. 명령어가 자기 일을 chat에 보고하지 않음 (가장 시급)

**증상**: `:check-canon`, `:canon-draft`, `:post-write`, `:pre-write` 모두 결과를 산출하지만 chat에는 generic audit summary만 노출. 핵심 결과는 파일 시스템에 묻혀 있음.

**구체 사례**:
- `:check-canon` 실행 → 45건 drift 발견 → chat에 한 줄도 안 나옴 → vibe coder 거짓 안심
- `:canon-draft` 실행 → `canonical-draft/helper-registry.v9.md` 새로 생성 → chat에 한 줄도 안 나옴 → 만들어진지도 모름
- `:post-write` 실행 → intent의 5개 lane 중 1개(plannedTypeEscapes)만 검사 → 코드 변경 없어도 "All clean" → 거짓 OK 사인

**해결 방향**: 각 명령어 chat 출력에 *"이 명령어의 핵심 결과 1~3줄"* 의무 노출. 일반 audit summary와 분리해서 명령어 고유 결과를 먼저.

### B. maintainer 어휘가 본문에 그대로 새어나옴

**증상**: JSON 경로(`checklist-facts.A6_circular_deps.sccCount`), FP id (FP22, FP23), bucket A/B/C, fan-in tier, drift 카테고리, escape kind 11종 — 모두 *"reserve unless asked"* 가 정책인데 기본 출력에 그대로 등장.

**구체 사례**:
- 1차 audit 응답에서 `checklist-facts.A6_circular_deps.sccCount = 0` 같은 점 표기를 본문에 직접 출력
- `:post-write` 출력에 11종 TS escape kind 모두 나열 (.mjs only 프로젝트인데도)
- `:refactor-plan` 출력에 "REVIEW_FIX 9 → 6", "bucket A", "FP 등록", "downgrade" 등 도구 어휘 잔존

**해결 방향**: lint/금지어 접근 대신 **AI 사고 anchor를 옮기는 프롬프트 지시**를 채택.

예시 anchor:
> *"이 결과는 코딩 처음 배우는 친구한테 설명한다고 생각해 주세요. 전문 용어 안 쓰고, 비유나 일상 표현으로 풀어서. 단, 도구가 100% 확신하지 못하는 건 '아마', '~인 것 같아요' 처럼 부드럽게 표시."*

이 방식이 lint보다 강한 이유:
- **긍정 프레이밍** — *해야 할 것*을 직접 말함 (lint는 *하지 말 것*만)
- **톤 + 어휘 동시에 잡힘** — 단어 검열이 아니라 AI 사고 모드 자체가 옮겨짐
- **자기 검증 가능** — AI가 출력 전에 *"이걸 친구가 이해할까?"* 스스로 체크
- **우회로 없음** — 안 막힌 단어가 기준이 아니라 *읽었을 때 느낌*이 기준
- **유지보수 0** — 새 도구 어휘 생겨도 자동 적용

**박을 위치 후보**:
- `SKILL.md` 머리말 — 모든 mode에 자동 적용 (1차 사용자 = vibe coder니까 결에 맞음)
- 또는 vibe-default mode routing — maintainer mode와 분리 가능

**캘리브레이션 옵션** (어른 사용자가 무시당한다 느끼지 않게):
- *"코딩 처음 배우는 친구한테 설명한다고 생각해주세요"*
- *"중학생도 이해할 수 있게"*
- *"전문 용어 안 쓰고, 비유나 일상 표현으로 풀어서"*

**위험요소**: 중요한 caveat가 단순화되어 사라질 수 있음 (예: *"Tier C means no consumer found, not definitely dead"* → *"이건 안 쓰는 거 같아요"*). 보완: anchor 프롬프트에 *"확신 정도를 부드럽게 표현하라"* 한 줄 같이.

### C. vibe / maintainer 페르소나 표지가 없음

**증상**: `:check-canon`, `:canon-draft` 처럼 maintainer-only 성격 명령어를 vibe coder가 잘못 누르면 위험한 거짓 안심까지 남음. 그런데 `:welcome`이나 routing에 페르소나 표지가 없음.

**해결 방향**: `:welcome` 선택지 또는 routing에서 *"이 명령어는 maintainer용입니다 — 도구 자체를 관리할 때 쓰세요"* 명시.

---

## 톤 예제집 (soft tone examples)

anchor 프롬프트(*"코딩 처음 배우는 친구한테 설명한다고 생각해 주세요"*)만으로는 추상적이에요. 다음 작업자(Codex)는 구체적 before/after 쌍을 모범으로 삼고 작업해 주세요. **딱딱한 톤이 기본 출력으로 나가지 않게**, 아래 예제들을 표준 참고로.

### 예 1 — drift 보고 (`:check-canon`)

**딱딱**:
```
Drift count: 45
- helper-registry: drift (32)
- topology: drift (13)
- type-ownership: clean (0)
- naming: clean (0)
```

**부드러움**:
> 4가지 약속 중 2개는 잘 지켜지고 있어요. 나머지 2개에서 총 45개 정도 변화가 있었네요. 대부분 *새 도우미 함수가 생겼다* 같은 좋은 신호예요. 약속 문서가 그저 못 따라잡은 것뿐이라, `canon-draft`로 정리하시면 돼요. 같이 해드릴까요?

### 예 2 — dead-export 발견 (`:audit`)

**딱딱**:
```
REVIEW_FIX: 9 findings (bucket A: 7, bucket B: 1, bucket C: 1)
- _lib/shape-hash.mjs:146 stableJson (bucket B, 5 internal uses)
- _lib/shape-hash.mjs:150 hashNormalizedShape (bucket A, 1 internal use)
- ... 7 more
```

**부드러움**:
> 한가할 때 들여다보면 좋을 자리 9개 정도 있어요. 급한 건 아니에요. 대부분 *"파일 안에서만 쓰는데 밖으로 나와 있는"* 함수들이거든요.
> - `_lib/shape-hash.mjs` 안에 4개 — 그 중 `stableJson`은 안에서 5번이나 쓰여서 일부러 노출한 것 같아요. 그대로 두는 게 좋겠어요.
> - 나머지는 진짜 안 쓰면 파일 안쪽 함수로 만들어도 되는 후보들이에요.

### 예 3 — post-write 보고

**딱딱**:
```
Any delta (silent-new counts):
- explicit any:                +0
- as any:                      +0
- as unknown as T:             +0
- ts-ignore:                   +0
- ... (11 lines, all +0)

Capability parity: ok
Scan-range parity: ok
```

**부드러움**:
> TypeScript 안전장치 새로 생긴 거 없어요. (이 프로젝트는 TS 안 쓰니까 항상 0이에요.)
>
> 그리고 내가 하기로 했던 일 — `_lib/json-utils.mjs` 에 `toStableJson` 함수 추가하기 — 아직 안 만들어졌네요. 깜빡한 거면 지금 만드시면 되고, 마음 바뀐 거면 그대로 둬도 돼요.

### 예 4 — canon-draft 생성

**딱딱**:
```
canonical-draft/helper-registry.v9.md created
helper-registry diffs: 32
- helper-added: 27
- helper-removed: 3
- label-changed: 2
```

**부드러움**:
> 약속 문서 새 초안을 만들었어요: `canonical-draft/helper-registry.v9.md`
>
> 도우미 함수 32개 변화를 반영했어요 — 27개는 새로 생긴 것, 3개는 사라진 것, 2개는 역할이 바뀐 거예요. 마음에 들면 `canonical/` 폴더로 옮기시면 정식 약속이 돼요.
>
> (참고: 이전 초안 v2~v8이 같이 쌓여있어요. 한 번 정리하시겠어요?)

### 예 5 — 확신을 부드럽게 (caveat 보존)

**딱딱**:
```
Tier C: no consumer found. Potentially dead. Resolver confidence: high.
```

**부드러움**:
> 이 함수는 어디서도 안 쓰이는 것 같아요. (도구가 100% 확신하는 건 아니에요 — 동적으로 부르는 곳이 있을 수도 있어서요.) 한 번 확인해 보시고, 진짜 안 써도 되겠다 싶으면 그때 지우셔도 돼요.

### 예 6 — pre-write 비슷한 이름 발견

**딱딱**:
```
NOT_OBSERVED for `toStableJson`
Semantic hint: stableJson at _lib/shape-hash.mjs (score: 2; matched tokens: stable, json)
```

**부드러움**:
> `toStableJson` 라는 이름은 아직 없어요. 근데 비슷한 이름이 있긴 해요 — `_lib/shape-hash.mjs` 에 `stableJson` 라는 함수가 있어요. 같은 일을 하는 함수일 수도 있으니 한번 보고 결정하시면 좋겠어요.

### 톤 디자인 원칙 (요약)

위 예제들에서 공통으로 나타나는 6가지:

1. **숫자에 맥락 같이** — *"45개"* 만 말고 *"4개 영역 중 2개에 총 45개 변화"*
2. **신호의 의미 해석해 주기** — *"helper-added 27"* 가 아니라 *"새 도우미 함수가 생긴 좋은 신호예요"*
3. **다음 행동 제안** — *"이러이러 하시면 돼요"* / *"같이 해드릴까요?"*
4. **확신 정도 부드럽게** — *"100% 확신은 못 해요"*, *"~인 것 같아요"*
5. **사용자 자율성 존중** — *"~해도 돼요"* / *"마음 바뀌면 그대로 둬도 OK"*
6. **경어체 + 친구한테 설명하듯** — 너무 사무적이지도, 너무 어린아이 대상도 아닌 중간 톤

→ 이 6원칙은 anchor 프롬프트와 함께 작업 시 머리에 두면 좋아요. 템플릿이나 코드 출력 만들 때 *"이 6가지 중 몇 개를 만족하나"* 자가 점검 기준으로도 활용 가능.

---

## 명령어별 수정 타겟

### `:welcome` — 표현 다듬기 (세션 중 일부 진행)

**파일**: `references/command-routing.md`의 `welcome` 섹션 영어 시드 문구

**현재**:
- "Check this repo now"
- "Check before I code"
- "Help me plan a gentle refactor"

**제안**:
- "See how my project is doing now"
- "I'm about to add or change a feature"
- "I want to tidy things up gradually"

### `:audit` (default) — JSON 경로 추방

**파일**: `SKILL.md`의 Output Contract 섹션, `templates/REVIEW_CHECKLIST_SHORT.md`

**규칙**: SHORT 모드 본문에 `.json` 확장자, 점 표기법, FP id 등장 금지. 증거는 *"증거는 `.audit/` 폴더에 있어요"* 한 줄로 갈음.

### `:pre-write` — 빈 의도 폴백 추가

**현 동작**: intent가 비거나 너무 얇으면 *"NOT_OBSERVED"* 같은 차가운 응답.

**제안**:
1. vibe coder의 *"뭐가 멋질까요?"* 류 질문에 대해 *"먼저 둘러봐 드릴게요"* 폴백 흐름
2. 또는 `:welcome`에 4번째 선택지 *"뭘 더 하면 좋을지 같이 생각해 줘"* 추가

**파일**: `references/command-routing.md`의 `pre-write` 섹션, 또는 새 mode 추가

### `:post-write` — 5개 lane 모두 검사 + chat 보고 (가장 큰 엔진 변경)

**현 동작**: 5개 intent lane 중 `plannedTypeEscapes` 1개만 검사.

**제안**:
1. `names`, `files`, `shapes`, `dependencies` lane 추가 검사
2. chat 본문에 *"내가 하기로 한 거 다 했나요?"* + *"안 하기로 한 게 늘었나요?"* 두 줄 답 의무
3. TS escape 11종은 .mjs only 프로젝트에서는 *"TS escape 0건"* 한 줄로 압축
4. 결론 한 줄 추가: *"전부 OK, 커밋해도 돼요"* / *"⚠️ 약속한 X가 없어요"*

**파일**: `_lib/post-write-delta.mjs`, `scripts/audit-repo.mjs` (post-write 흐름)

### `:check-canon` — chat에 drift 요약 의무 노출

**현 동작**: 45건 drift 발견해도 chat에는 일반 audit summary만 노출.

**제안**:
1. chat 본문에 *"4개 영역 중 N개 깨끗, M개에서 총 K건 어긋남"* 한 줄 의무
2. drift 카테고리(`helper-added`, `oversize-changed`, `cross-edge-added` 등)를 vibe coder 말로 번역
3. drift 의미 톤 가이드: 27개 helper-added는 *"건강한 발전"* 신호 — *"이건 보통 좋은 신호예요. canon-draft로 따라잡으세요"* 같은 해설
4. *"이 명령은 maintainer용입니다"* 머리말 추가

**파일**: `scripts/audit-repo.mjs` (check-canon chat output), `references/command-routing.md`의 `check-canon` 섹션

### `:canon-draft` — chat에 생성 결과 의무 노출 + 정리 흐름

**현 동작**: 새 draft 만들어도 chat에는 일반 audit summary만 노출. `canonical-draft/`에 v2~v13까지 40+개 누적.

**제안**:
1. chat 본문에 *"새 초안을 만들었어요: `canonical-draft/helper-registry.v9.md` (32건 반영)"* 의무 노출
2. *"기존 v2~v8 초안 정리하시겠어요?"* 안내 또는 자동 prune 정책
3. *"promote는 직접 `canonical/`로 옮겨야 해요"* 흐름 안내
4. *"이 명령은 maintainer용입니다"* 머리말 추가

**파일**: `scripts/audit-repo.mjs` (canon-draft chat output), `_lib/audit-canon-draft.mjs`

### `:refactor-plan` — 번역 표 강제 + 템플릿 예시 vibe coder 톤

**현 동작**: 정책에 번역 표 있지만 권고 수준. 템플릿 예시 자체가 maintainer 톤.

**제안**:
1. **anchor 프롬프트를 정책 머리에 박기** — *"코딩 처음 배우는 친구한테 설명한다고 생각해 주세요"* (B 패턴 해결 방향 참조)
2. `templates/refactor-plan-template.md`의 예시를 vibe coder 톤으로 다시 작성 — AI가 모범으로 삼을 수 있게
3. *"Ask the coding agent"* 블록도 한국어/평이한 표현으로 (지금은 영어 + 도구 어휘)
4. 기존 번역 표(7개 항목)는 *참고 사전*으로 유지 — 강제 규칙 아님. anchor 프롬프트가 이미 같은 일을 더 강하게 함

**파일**: `references/refactor-plan-policy.md`, `templates/refactor-plan-template.md`

---

## 작업 우선순위 (제안)

1. **A 패턴 해결 (시급)** — `:check-canon`, `:canon-draft`, `:post-write` chat 보고 의무화
2. **C 패턴 해결 (간단)** — `:welcome`/routing에 페르소나 표지 한 줄 추가
3. **`:welcome` 표현 다듬기** — 영어 시드 문구 업데이트
4. **B 패턴 해결** — anchor 프롬프트를 SKILL.md 머리에 추가 + 템플릿 예시 vibe coder 톤으로 재작성 (lint 접근 폐기 — 우회로 생김)
5. **`:post-write` 핵심 질문 재정의** — 가장 큰 엔진 변경 작업

---

## 확인된 사실 (grounded 항목 — 다음 작업자 참고용)

세션 중 실제로 돌린 audit 결과:

- **현재 저장소 상태**: 215 파일, 의존성 순환 0건, parser 깨끗, blind zone 없음
- **dead-export REVIEW_FIX 9건**: 대부분 `_lib/shape-hash.mjs`, `_lib/canon-draft-topology.mjs`, `_lib/self-audit-excludes.mjs`, `_lib/p6-measurement.mjs`, `_lib/shape-index-artifact.mjs`, `_lib/test-paths.mjs`. maintainer scripts/_engine 제외 영향 가능 — FP 가능성 검토 필요.
- **canon drift 45건**: helper-registry 32건 (대부분 helper-added — 건강한 발전), topology 13건 (oversize 변동 + cross-edge 1건). type-ownership / naming 깨끗.
- **silent catch**: 46건 anonymous (E2_silent_catch). `_lib/audit-check-canon.mjs:52` 부근 watch.

**증거 위치**:
- `.audit/manifest.json`
- `.audit/fix-plan.json`
- `.audit/canon-drift.json`, `.audit/canon-drift.helper-registry.md`, `.audit/canon-drift.topology.md`
- `.audit/post-write-delta.latest.json`
- `.audit/pre-write-advisory.latest.json`

---

## 자기 비평 메모 (참고)

세션 중 **assistant가 첫 audit 응답에서 직접 위 B 패턴을 어김**. JSON 점 표기법, REVIEW_FIX, bucket A 같은 maintainer 어휘를 본문에 그대로 출력. SKILL.md의 *"keep raw JSON ... in reserve unless proof is requested"* 정책 위반.

→ 강제 규칙 없는 정책은 LLM이 충실히 따라가지 못함. 다만 lint/금지어 접근은 우회로가 생기는 한계가 있어, **AI 사고 anchor를 옮기는 프롬프트 지시**(예: *"코딩 처음 배우는 친구한테 설명한다고 생각해 주세요"*) 방향이 더 효과적이라는 결론에 도달 (메타-패턴 B 해결 방향 참조).

`templates/refactor-plan-template.md`의 공식 예시("self-audit path", "skill mirrors", "scan-scope helpers" 등) 자체가 maintainer 톤이라, AI가 충실히 따라가도 vibe coder에게 차가운 출력이 나옴. **템플릿 예시부터 손봐야 모범이 됨.**

---

## 한 줄 요약

> 이 도구는 *의도는 vibe coder, 실행은 maintainer*. 격차의 원인은 (A) 명령어 자기 결과 미보고, (B) 도구 어휘 자유 통과, (C) 페르소나 표지 부재. **A → C → B** 순으로 손대면 같은 시간 대비 가장 큰 친화도 향상.
