# Change tw spacing
Change tw spacer is a simple cli tool that allows you to update the `--spacing` variable in your `tailwind v4.x` config **without changing the actual spacing accross the project**. The goal here is to *refactor* existing code, without changing what its output looks like. An example of how this works is as follows:

**Before change tw spacing**
```css
/* index.css */

@theme {
  ...
  --spacing: 4px;
  ...
}
```

```tsx
/* home.tsx */

<h1 className="mb-20">
  Hello World
</h1>
```

**Before after change tw spacing**
```css
/* index.css */

@theme {
  ...
  --spacing: 4px;
  ...
}
```

```tsx
/* home.tsx */

<h1 className="mb-5">
  Hello World
</h1>
```

## Why this exists?

Some novice frontend developers will sometimes look at the spacing variable and think "oh, why don't I just set the --spacing to 1px?" Which initially seems like a good idea until you run into a situation where some library just assumes you have tailwind's default spacing. I ran into such a problem particularly with [ReUI](https://reui.io/). I was working on a project started by interns, and ReUI was incredibly useful, but I had to multiply all the spacing by 4 for each component I installed.

My solution was to revert to the standard of `0.25rem` spacing, but doing that manually is a lot of work, and I couldn't trust an LLM or an Agent with this job, and even if I could it would require quite a lot of tokens for a project with 200+ files, and I was out here using free plans.

## Important notes

A number of things are hardcoded in this project, because the main goal was for it to solve this problem once. However should this project receive more care or work, these will be made configurable:

- The utility classes that the program searches for are written in code in [search.rs](src/search.rs) in the `get_classes_regex` function. Please note that some utility classes can only be positive while others can be negative.
- The types of files searched for usage of those utility classes are defined in the `is_tw_file` function in [files.rs](src/files.rs).
- The directories to be ignored aren't defined by .gitignore (which would probably be the right way to go about this), but rather in [files.rs](src/files.rs).
