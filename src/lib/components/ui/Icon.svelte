<script>
  import {
    IconChevronRight,
    IconActivity,
    IconAlertTriangle,
    IconBalloon,
    IconBarbell,
    IconBolt,
    IconBuildingHospital,
    IconCalendar,
    IconChartBar,
    IconCheck,
    IconClipboardList,
    IconClock,
    IconDownload,
    IconEye,
    IconHeart,
    IconListDetails,
    IconUser,
    IconMedal,
    IconInfoCircle,
    IconPencil,
    IconPill,
    IconPlus,
    IconSettings,
    IconRefresh,
    IconSquareRoundedX,
    IconStethoscope,
    IconTag,
    IconTrash,
    IconUserPlus,
    IconDatabase,
    IconFileText,
    IconNote,
  } from '@tabler/icons-svelte';

  export let name = '';
  export let size = 22;
  export let stroke = 2;
  export let className = '';
  export { className as class };

  const icons = {
    list: IconListDetails,
    clipboard: IconClipboardList,
    chevronRight: IconChevronRight,
    add: IconPlus,
    stats: IconChartBar,
    heart: IconHeart,
    hospital: IconBuildingHospital,
    view: IconEye,
    edit: IconPencil,
    delete: IconTrash,
    refresh: IconRefresh,
    download: IconDownload,
    close: IconSquareRoundedX,
    settings: IconSettings,
    calendar: IconCalendar,
    timer: IconClock,
    pill: IconPill,
    tag: IconTag,
    check: IconCheck,
    medal: IconMedal,
    stethoscope: IconStethoscope,
    balloon: IconBalloon,
    activity: IconActivity,
    bolt: IconBolt,
    strength: IconBarbell,
    userPlus: IconUserPlus,
    alert: IconAlertTriangle,
    info: IconInfoCircle,
    user: IconUser,
    database: IconDatabase,
    file: IconFileText,
    note: IconNote,
  };

  $: Component = icons[name];
</script>

{#if Component}
  <Component size={size} strokeWidth={stroke} class={className} />
{:else}
  <span class={className}>?</span>
{/if}
