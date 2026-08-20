<script lang="ts">
  import * as AlertDialog from "$lib/components/ui/alert-dialog";

  let {
    open = $bindable(false),
    title,
    consequence,
    keepLabel = "Keep instance",
    confirmLabel = "Delete",
    onconfirm,
  }: {
    open?: boolean;
    title: string;
    consequence: string;
    keepLabel?: string;
    confirmLabel?: string;
    onconfirm: () => void;
  } = $props();
</script>

<AlertDialog.Root bind:open>
  <AlertDialog.Content class="max-w-sm">
    <AlertDialog.Header>
      <AlertDialog.Title>{title}</AlertDialog.Title>
      <AlertDialog.Description>
        {consequence} This cannot be undone.
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel variant="outline">{keepLabel}</AlertDialog.Cancel>
      <AlertDialog.Action variant="destructive" onclick={onconfirm}
        >{confirmLabel}</AlertDialog.Action
      >
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
