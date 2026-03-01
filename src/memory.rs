use x86_64::{
    structures::paging::{PageTable, OffsetPageTable, PhysFrame, Size4KiB, FrameAllocator, PageTableFlags},
    VirtAddr, PhysAddr,
};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use spin::Mutex;
use crate::serial_println;

// Stockage global pour le shell et les drivers
pub static MAPPER: Mutex<Option<OffsetPageTable<'static>>> = Mutex::new(None);
pub static PHYS_MEM_OFFSET: Mutex<Option<VirtAddr>> = Mutex::new(None);

/// Initialise le système de mémoire global.
pub unsafe fn init_global(physical_memory_offset: VirtAddr) {
    let level_4_table = active_level_4_table(physical_memory_offset);
    
    // On initialise le mapper
    let mapper = OffsetPageTable::new(level_4_table, physical_memory_offset);
    *MAPPER.lock() = Some(mapper);
    
    // On sauvegarde l'offset pour force_user_access plus tard
    *PHYS_MEM_OFFSET.lock() = Some(physical_memory_offset);
}

/// Récupère la table de niveau 4 active.
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    &mut *page_table_ptr
}

/// Force l'accès User Mode en parcourant toute la hiérarchie (P4 -> P1).
/// Retourne true si toute la hiérarchie a pu être déverrouillée, false sinon.
pub unsafe fn force_user_access(physical_memory_offset: VirtAddr, addr: VirtAddr) -> bool {
    let mut table = active_level_4_table(physical_memory_offset);

    let indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];

    for (level, &index) in indexes.iter().enumerate() {
        let entry = &mut table[index];

        // Vérifier que l'entrée est présente AVANT de modifier les flags
        if !entry.flags().contains(PageTableFlags::PRESENT) {
            // L'entrée intermédiaire n'est pas présente - on ne peut pas continuer
            // mais on log un Warning plutôt que de crasher
            serial_println!("[MEMORY] Warning: Page table entry at level {} not present for addr {:#x}", level, addr);
            return false;
        }

        let mut flags = entry.flags();
        flags |= PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE;
        entry.set_flags(flags);

        if level < 3 {
            // On doit accéder à la prochaine table
            match entry.frame() {
                Ok(frame) => {
                    let next_table_virt = physical_memory_offset + frame.start_address().as_u64();
                    table = &mut *(next_table_virt.as_mut_ptr::<PageTable>());
                }
                Err(_) => {
                    serial_println!("[MEMORY] Error: Could not get frame at level {}", level);
                    return false;
                }
            }
        }
    }

    use x86_64::instructions::tlb;
    tlb::flush(addr);
    true
}

// --- ALLOCATEUR DE FRAMES ---

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator { memory_map, next: 0 }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}