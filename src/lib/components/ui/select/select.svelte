<script lang="ts">
	import type { HTMLSelectAttributes } from "svelte/elements";
	import { cn } from "$lib/utils.js";
	import { ChevronDown } from "lucide-svelte";

	type Props = HTMLSelectAttributes & {
		ref?: HTMLSelectElement | null;
		value?: string;
	};

	let {
		ref = $bindable(null),
		value = $bindable(),
		class: className,
		children,
		...restProps
	}: Props = $props();
</script>

<div class="relative w-full">
	<select
		bind:this={ref}
		bind:value
		class={cn(
			"border-input bg-background selection:bg-primary dark:bg-input/30 selection:text-primary-foreground ring-offset-background placeholder:text-muted-foreground flex h-9 w-full min-w-0 rounded-md border px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm appearance-none cursor-pointer pr-10",
			"focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]",
			"aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
			className
		)}
		{...restProps}
	>
		{@render children?.()}
	</select>
	<ChevronDown class="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400 pointer-events-none" />
</div>
