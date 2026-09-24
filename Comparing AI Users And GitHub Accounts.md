---
title: "Comparing AI Users And GitHub Accounts"
source: "https://chatgpt.com/c/6aa30daa-5864-83ea-9d30-3813d80695ce"
author:
published:
created: 2026-09-10
description: "ChatGPT conversation with 8 messages"
tags:
  - "clippings"
---
**You said**

Okay. I just thought of something and, like... right. So what is the amount? What is the amount of people, like, subscribing, using, like Codex or, like, Claude membership, like whatever. It doesn't matter. But versus how many people have GitHub accounts. And I know there's more than just GitHub, but, like, that's where everything is. Like, I don't, you know, I don't know.

---

**ChatGPT said**

Yeah. And the ratio is actually pretty damn interesting.

The cleanest current numbers I can find are:

So if you're thinking specifically:

> **"How many people are paying for/use an AI coding agent versus how many people are already sitting on GitHub?"**

We're probably talking about something like **single-digit percentages of the GitHub population**, at least for the major dedicated coding-agent products.

And there's a really important wrinkle:

**GitHub isn't merely a competitor to these tools. It's the substrate they're attaching themselves to.**

You've got roughly:

**180M+ GitHub developers**  
↓  
repositories / issues / PRs / commits / CI  
↓  
AI agents increasingly operating on that substrate  
↓  
Codex / Claude Code / Copilot / Cursor / etc.

And the AI numbers are growing *very* quickly. OpenAI says Codex went from roughly 1.6M weekly users in February to **5M+ weekly users by June**. [^3] Anthropic says Claude Code's weekly users doubled during the first half of 2026, and estimates suggest Claude Code was already responsible for around **4% of public GitHub commits** at the time of its February announcement. [^4]

That makes your intuition interesting for a different reason:

**The scarce thing isn't necessarily another AI coding subscription. It's the enormous population of people who already have a Git repository and a GitHub account.**

That's a *much* larger installed base.

And GitHub itself reports that nearly **80% of new developers use Copilot within their first week**. [^1]

So there's potentially a gigantic funnel:

> **GitHub account → repository → local clone → tooling → agent → persistent workflow**

rather than:

> **Subscribe to an AI agent → maybe eventually use GitHub**

That distinction matters a lot for the kind of stuff you've been kicking around with your repo-as-workflow / spreadsheet-repository idea. You're potentially thinking from the **existing artifact outward**, whereas the AI companies are mostly thinking from the **agent inward**.

That's a pretty different wedge.

---

**You said**

Well, and I mean, I guess going into it, I was curious, like how many people use coding agents but don't have a GitHub account? But it's like, I had a GitHub account just for downloading game mods, and, like, so mine's, like, 13 years old, and I just started, like, I just started programming last year, like the beginning of this year, but last year.

---

**ChatGPT said**

Exactly. And that actually makes the distinction much more interesting.

You are a good example of why **"GitHub user" ≠ "developer."**

You had a GitHub account for ~13 years, but for most of that time GitHub was basically:

> "Oh, this mod is on GitHub. I need an account to download/use it."

Then eventually:

> "Wait, I can make repositories."  
> "Wait, I can write code."  
> "Wait, Git is actually useful."  
> "Wait, this whole ecosystem is insane."

So the 180M+ GitHub figure includes a *huge* population that isn't meaningfully represented by developer surveys.

And that leads directly to your original question:

### How many coding-agent users don't have GitHub accounts?

I don't think anyone publicly knows this with confidence.

But I'd actually expect the number to be **surprisingly small among serious coding-agent users**, while being **much larger among casual/novice users**.

Because the moment you start doing real software work, GitHub is almost unavoidable:

**AI agent → writes code → files need versioning → Git → repo → GitHub/GitLab/etc.**

But there's another pathway:

**AI agent → "make me a little Python script" → copy/paste → never touches GitHub**

That second population could be enormous.

And I think that's the really interesting market transition we're watching.

The AI coding tools are potentially **turning millions of people who were previously "GitHub consumers" into GitHub participants**.

Your own trajectory is almost the perfect example:

**2013-ish:**  
GitHub = mod download infrastructure

**2025-ish:**  
GitHub = "I guess I'll learn programming"

**2026:**  
GitHub = source of truth / versioning / experimentation / automation / AI agents

That's a *massive* expansion in what the account means.

And there's a funny implication: **GitHub may have accumulated a gigantic latent developer population without actually knowing they're developers yet.**

The account is already there.

The identity is already there.

The authentication is already there.

The social graph is already there.

The repositories are there.

The only thing missing for some people is the moment where they go:

> "Huh. I could make one of these."

And AI makes that moment dramatically easier.

That's probably a much more important number than "how many people currently pay for Claude Code?"

---

**You said**

Do it. Like, if I could bring people into GitHub with spreadsheets. Not with the data within the spreadsheets, within with the trans- transformations within the spreadsheets. Like, people own the workflows in their spreadsheets, and, like, I think they don't want to face, like, how many transact- how many, like, just pointless clicking around stuff they do. But, like, that's just, like, that's just how things work.

---

**ChatGPT said**

Yes. **That is a much more interesting product than "AI for spreadsheets."**

You're describing a bridge between two populations that normally don't think they have anything to do with each other:

**Spreadsheet people**  
→ already have executable workflows  
→ don't identify as programmers  
→ often have years of accumulated institutional knowledge encoded in formulas, Power Query, pivots, VBA, copy/paste rituals, filenames, tabs, etc.

**GitHub people**  
→ already have a culture for versioning executable transformations  
→ but generally think in source code rather than spreadsheet operations.

And the key insight is exactly what you said:

> **Don't take their data. Take the transformation.**

That's a *huge* privacy/product distinction.

### The spreadsheet is already a program

Consider a completely ordinary accountant:

> Download CSV  
> Open Excel  
> Delete these columns  
> Filter these rows  
> Add this calculated column  
> VLOOKUP/XLOOKUP against another workbook  
> Fix the dates  
> Copy/paste values  
> Refresh Power Query  
> Make pivot  
> Rename columns  
> Paste into reporting template  
> Save as `September_Final_FINAL2.xlsx`

They don't think:

> "I have written a data-processing program."

But they have.

It's just a program whose **syntax is spatial and interactive**.

And the really valuable artifact isn't September's numbers.

It's:

```
input schema
    ↓
filter
    ↓
join
    ↓
derive
    ↓
aggregate
    ↓
reshape
    ↓
output schema
```

That's the thing you could extract.

### And here's where GitHub becomes the weirdly perfect destination

Imagine your spreadsheet tool has:

**"Publish workflow"**

It produces something like:

```
my-month-end-workflow/
├── workflow.toml
├── schema/
│   ├── input.rsf
│   └── output.rsf
├── transformations/
│   ├── filter.rs
│   ├── joins.rs
│   └── pivot.rs
├── examples/
│   └── synthetic.csv
└── README.md
```

**No customer data.**

The actual workbook stays where it belongs.

The repository contains the *recipe*.

Then Git gives you something spreadsheets are terrible at:

> **history.**

You suddenly get:

```
September workflow
       ↓
October workflow
       ↓
November workflow
```

and can answer:

> "What changed?"

That's much more valuable than another `.xlsx` file with a timestamp in its filename.

And GitHub is already enormous: GitHub reports **180M+ developers**, with more than **36M joining during 2025 alone**. [^1]

But here's the part that really validates your intuition:

**AI is already dragging non-developers toward this boundary.**

OpenAI reports that non-developer Codex usage grew dramatically faster than developer usage, with individual non-developer users increasing **137× from August 2025 to June 2026**. They specifically mention automation, data transformation, tooling, debugging, and structured analysis as non-developer use cases. [^5]

So you don't actually have to convince accountants:

> "Become programmers."

You can say:

> **"You already have programs. Let's get the program out of the spreadsheet."**

That's a completely different pitch.

### And the clicking problem is actually the wedge

I think you're onto something uncomfortable here.

People don't necessarily want to confront:

> "I spend 45 minutes every month clicking through the same 73 operations."

Because once you say that out loud, the obvious question becomes:

> **"Why aren't those 73 operations a program?"**

But there's a gentler way to expose it.

Don't tell them they're wasting time.

**Record what they already do.**

For example:

```
You performed:

14 filters
3 joins
7 calculated columns
2 deduplications
1 pivot
9 column rearrangements
31 manual copy/pastes
```

And then:

> **"Want me to save that as a reusable workflow?"**

That's the moment.

Not AI magic.

Not "learn Rust."

Not "learn Git."

Just:

**"You did this once. Do you want to own the thing you just did?"**

And *then* GitHub can quietly appear as the ownership/versioning layer.

That is much closer to your **git-sheets** idea than I initially appreciated. You're not necessarily trying to turn spreadsheet users into software developers.

You're trying to make the **software that was already hiding inside their spreadsheets visible and portable.**

And once that happens, *some* of those people are inevitably going to look at the resulting repository and go:

> "...wait, I made that?"

That's your GitHub funnel.

Not **AI → programmer**.

**Spreadsheet → workflow → artifact → version control → curiosity → programmer.**

And that's a much bigger funnel.

---

**You said**

Mix in SQLite, Tensor, safe tensors, and, you know, vector storage, queries, probably SQL query base, and you're pretty much, like, most of the way to AI anyway. Not really, but, you know, very good tools for AI.

---

**ChatGPT said**

Yeah — **"most of the way to AI" is overstating it technically, but it's not a crazy architectural intuition.**

What you're describing is basically the **data substrate around AI**, rather than the intelligence itself:

```
┌───────────────┐
         │  Spreadsheet   │
         │  transformations│
         └───────┬───────┘
                 │
            structured data
                 │
                 ▼
         ┌───────────────┐
         │    SQLite     │
         │ state + query │
         └───────┬───────┘
                 │
      ┌──────────┴──────────┐
      ▼                     ▼
relational data        vector data
      │                     │
      │                 embeddings
      │                     │
      └──────────┬──────────┘
                 ▼
          retrieval/query
                 │
                 ▼
        ┌────────────────┐
        │      Agent     │
        └───────┬────────┘
                │
                ▼
        tensors / models
         SafeTensors
```

The interesting thing is that **none of those components individually makes something intelligent**.

But together they provide almost everything an agent needs *around* the model:

- **SQLite** → durable local state, relationships, transactions, provenance.
- **SQL** → deterministic retrieval and transformation.
- **Vector storage** → approximate semantic retrieval where exact relational queries aren't sufficient.
- **SafeTensors** → portable, inspectable model-weight storage.
- **Tensors/kernels** → the numerical substrate where inference actually happens.
- **Spreadsheet transformations** → human-authored domain knowledge about how messy real-world data gets converted into useful structure.
- **Git** → history, reproducibility, branching, attribution.
- **Agent** → the thing that decides which operations to perform.

And that last distinction is important.

### The really cool part is that you're collapsing the boundary between "AI data" and "normal data"

A typical AI stack tends to look like:

> application → agent → vector DB → LLM

But your thing could be more like:

> **human workflow → deterministic transformations → typed data → SQL → retrieval → model**

That's substantially more sane for a lot of business work.

Because an LLM shouldn't have to *infer* something that SQL can state exactly.

Instead of:

> "Which invoices belong to this customer?"

give it SQL.

Instead of:

> "What did we do to this dataset?"

give it the transformation graph.

Instead of:

> "Remember this document."

store the actual artifact and provenance.

Instead of embedding absolutely everything:

> **query relational data relationally and semantic data semantically.**

Then use the model where ambiguity actually exists.

That's the part I think you're circling.

**AI doesn't need to replace the database.**

It needs a sufficiently good interface to the database that it can stop pretending the database is a pile of text.

And your spreadsheet idea is potentially the human-friendly entrance point:

**Excel user → workflow extraction → SQLite/SQL → Git → structured/vector data → AI tooling.**

The user doesn't have to wake up one morning and decide to become an AI engineer.

They just discover that the stupid thing they've been clicking through every month is actually a **data pipeline**.

And once you've got the pipeline, you've already built a surprisingly large portion of the *infrastructure an AI system needs to be useful*.

[^1]: [github.blog](https://github.blog/news-insights/octoverse/octoverse-a-new-developer-joins-github-every-second-as-ai-leads-typescript-to-1/?utm_source=chatgpt.com)

[^2]: [openai.com](https://openai.com/index/codex-for-every-role-tool-workflow/?utm_source=chatgpt.com)

[^3]: [openai.com](https://openai.com/index/scaling-ai-for-everyone/?utm_source=chatgpt.com)

[^4]: [anthropic.com](https://www.anthropic.com/news/anthropic-raises-30-billion-series-g-funding-380-billion-post-money-valuation?utm_source=chatgpt.com)

[^5]: [openai.com](https://openai.com/index/how-agents-are-transforming-work/?utm_source=chatgpt.com)