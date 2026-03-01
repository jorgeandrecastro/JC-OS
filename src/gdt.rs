use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub fn get_kernel_code_selector() -> SegmentSelector { GDT.1.kernel_code_selector }
pub fn get_kernel_data_selector() -> SegmentSelector { GDT.1.kernel_data_selector }
pub fn get_user_code_selector() -> SegmentSelector { GDT.1.user_code_selector }
pub fn get_user_data_selector() -> SegmentSelector { GDT.1.user_data_selector }

/// Retourne le sommet de la pile de privilège (Ring 0) définie dans le TSS
pub fn get_tss_stack_ptr() -> VirtAddr {
    TSS.privilege_stack_table[0]
}
lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        // Pile pour les fautes graves (Double Fault)
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            stack_start + STACK_SIZE
        };
        // TRÈS IMPORTANT : Pile de privilège pour le passage Ring 3 -> Ring 0
        // C'est ici que le CPU bascule quand un user fait un syscall ou une interruption
        tss.privilege_stack_table[0] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            stack_start + STACK_SIZE
        };
        tss
    };
}
lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        
        // Index 1 : Kernel Code (0x08)
        let kernel_code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        // Index 2 : Kernel Data (0x10)
        let kernel_data_selector = gdt.add_entry(Descriptor::kernel_data_segment());

        // Index 3 : User Code (0x18 -> 0x1B avec RPL 3)
        let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());
        // Index 4 : User Data (0x20 -> 0x23 avec RPL 3)
        let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
        
        // Index 5 & 6 : TSS (Le TSS prend DEUX slots en 64 bits !)
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

        (gdt, Selectors { 
            kernel_code_selector, 
            kernel_data_selector, 
            user_code_selector, 
            user_data_selector, 
            tss_selector 
        })
    };
}
struct Selectors {
    kernel_code_selector: SegmentSelector,
    kernel_data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment, DS};

    GDT.0.load();
   unsafe {
    CS::set_reg(GDT.1.kernel_code_selector);
    DS::set_reg(GDT.1.kernel_data_selector);
    // Optionnel mais pro pour JC-OS :
    // ES::set_reg(GDT.1.kernel_data_selector);
    // SS::set_reg(GDT.1.kernel_data_selector);
    load_tss(GDT.1.tss_selector);
}
    
    crate::serial_println!("[GDT] Loaded with Ring 3 Support");
}