use atspi::Role;
use core::fmt;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};
use serde::{Deserialize, Serialize};

#[derive(Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleSet(u128, u8);

impl RoleSet {
	pub const EMPTY: RoleSet = RoleSet(0, 0);
	pub const ALL: RoleSet = RoleSet(u128::MAX, u8::MAX);

	const fn from_role(role: Role) -> Self {
		let (low, high) = role_bits(role);
		RoleSet(low, high)
	}
	fn contains(self, other: RoleSet) -> bool {
		(self & other) == other
	}
	fn role_iter(self) -> impl Iterator<Item = Role> {
		(0..u128::BITS)
			.filter(move |i| (self.0 >> i) & 0x1 == 1)
			.filter_map(|i| Role::try_from(i).ok())
			.chain((0..u8::BITS)
				.filter(move |i| (self.1 >> i) & 0x1 == 1)
				.map_while(|i| Role::try_from(i + u128::BITS).ok()))
	}
}

impl fmt::Debug for RoleSet {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_set().entries(self.role_iter()).finish()
	}
}

impl From<Role> for RoleSet {
	fn from(r: Role) -> Self {
		let (low, high) = role_bits(r);
		RoleSet(low, high)
	}
}

impl BitAndAssign<RoleSet> for RoleSet {
	// Required method
	fn bitand_assign(&mut self, rhs: RoleSet) {
		self.0 &= rhs.0;
		self.1 &= rhs.1;
	}
}
impl BitAndAssign<Role> for RoleSet {
	// Required method
	fn bitand_assign(&mut self, rhs: Role) {
		*self &= RoleSet::from_role(rhs);
	}
}
impl BitOrAssign<RoleSet> for RoleSet {
	// Required method
	fn bitor_assign(&mut self, rhs: RoleSet) {
		self.0 |= rhs.0;
		self.1 |= rhs.1;
	}
}
impl BitOrAssign<Role> for RoleSet {
	// Required method
	fn bitor_assign(&mut self, rhs: Role) {
		*self |= RoleSet::from_role(rhs);
	}
}

impl BitAnd<RoleSet> for RoleSet {
	type Output = RoleSet;
	// Required method
	fn bitand(self, rhs: RoleSet) -> Self::Output {
		RoleSet(self.0 & rhs.0, self.1 & rhs.1)
	}
}

impl BitAnd<Role> for RoleSet {
	type Output = RoleSet;
	// Required method
	fn bitand(self, rhs: Role) -> Self::Output {
		self & RoleSet::from_role(rhs)
	}
}
impl BitOr<RoleSet> for RoleSet {
	type Output = RoleSet;
	// Required method
	fn bitor(self, rhs: RoleSet) -> Self::Output {
		RoleSet(self.0 | rhs.0, self.1 | rhs.1)
	}
}
impl BitOr<Role> for RoleSet {
	type Output = RoleSet;
	// Required method
	fn bitor(self, rhs: Role) -> Self::Output {
		self | RoleSet::from_role(rhs)
	}
}

impl Not for RoleSet {
	type Output = RoleSet;
	fn not(self) -> RoleSet {
		RoleSet(!self.0, !self.1)
	}
}

#[allow(clippy::too_many_lines)]
const fn role_bits(r: Role) -> (u128, u8) {
	match r {
		Role::Invalid => (0, 0),
		Role::AcceleratorLabel => (1 << 1, 0),
		Role::Alert => (1 << 2, 0),
		Role::Animation => (1 << 3, 0),
		Role::Arrow => (1 << 4, 0),
		Role::Calendar => (1 << 5, 0),
		Role::Canvas => (1 << 6, 0),
		Role::CheckBox => (1 << 7, 0),
		Role::CheckMenuItem => (1 << 8, 0),
		Role::ColorChooser => (1 << 9, 0),
		Role::ColumnHeader => (1 << 10, 0),
		Role::ComboBox => (1 << 11, 0),
		Role::DateEditor => (1 << 12, 0),
		Role::DesktopIcon => (1 << 13, 0),
		Role::DesktopFrame => (1 << 14, 0),
		Role::Dial => (1 << 15, 0),
		Role::Dialog => (1 << 16, 0),
		Role::DirectoryPane => (1 << 17, 0),
		Role::DrawingArea => (1 << 18, 0),
		Role::FileChooser => (1 << 19, 0),
		Role::Filler => (1 << 20, 0),
		Role::FocusTraversable => (1 << 21, 0),
		Role::FontChooser => (1 << 22, 0),
		Role::Frame => (1 << 23, 0),
		Role::GlassPane => (1 << 24, 0),
		Role::HTMLContainer => (1 << 25, 0),
		Role::Icon => (1 << 26, 0),
		Role::Image => (1 << 27, 0),
		Role::InternalFrame => (1 << 28, 0),
		Role::Label => (1 << 29, 0),
		Role::LayeredPane => (1 << 30, 0),
		Role::List => (1 << 31, 0),
		Role::ListItem => (1 << 32, 0),
		Role::Menu => (1 << 33, 0),
		Role::MenuBar => (1 << 34, 0),
		Role::MenuItem => (1 << 35, 0),
		Role::OptionPane => (1 << 36, 0),
		Role::PageTab => (1 << 37, 0),
		Role::PageTabList => (1 << 38, 0),
		Role::Panel => (1 << 39, 0),
		Role::PasswordText => (1 << 40, 0),
		Role::PopupMenu => (1 << 41, 0),
		Role::ProgressBar => (1 << 42, 0),
		Role::PushButton => (1 << 43, 0),
		Role::RadioButton => (1 << 44, 0),
		Role::RadioMenuItem => (1 << 45, 0),
		Role::RootPane => (1 << 46, 0),
		Role::RowHeader => (1 << 47, 0),
		Role::ScrollBar => (1 << 48, 0),
		Role::ScrollPane => (1 << 49, 0),
		Role::Separator => (1 << 50, 0),
		Role::Slider => (1 << 51, 0),
		Role::SpinButton => (1 << 52, 0),
		Role::SplitPane => (1 << 53, 0),
		Role::StatusBar => (1 << 54, 0),
		Role::Table => (1 << 55, 0),
		Role::TableCell => (1 << 56, 0),
		Role::TableColumnHeader => (1 << 57, 0),
		Role::TableRowHeader => (1 << 58, 0),
		Role::TearoffMenuItem => (1 << 59, 0),
		Role::Terminal => (1 << 60, 0),
		Role::Text => (1 << 61, 0),
		Role::ToggleButton => (1 << 62, 0),
		Role::ToolBar => (1 << 63, 0),
		Role::ToolTip => (1 << 64, 0),
		Role::Tree => (1 << 65, 0),
		Role::TreeTable => (1 << 66, 0),
		Role::Unknown => (1 << 67, 0),
		Role::Viewport => (1 << 68, 0),
		Role::Window => (1 << 69, 0),
		Role::Extended => (1 << 70, 0),
		Role::Header => (1 << 71, 0),
		Role::Footer => (1 << 72, 0),
		Role::Paragraph => (1 << 73, 0),
		Role::Ruler => (1 << 74, 0),
		Role::Application => (1 << 75, 0),
		Role::Autocomplete => (1 << 76, 0),
		Role::Editbar => (1 << 77, 0),
		Role::Embedded => (1 << 78, 0),
		Role::Entry => (1 << 79, 0),
		Role::CHART => (1 << 80, 0),
		Role::Caption => (1 << 81, 0),
		Role::DocumentFrame => (1 << 82, 0),
		Role::Heading => (1 << 83, 0),
		Role::Page => (1 << 84, 0),
		Role::Section => (1 << 85, 0),
		Role::RedundantObject => (1 << 86, 0),
		Role::Form => (1 << 87, 0),
		Role::Link => (1 << 88, 0),
		Role::InputMethodWindow => (1 << 89, 0),
		Role::TableRow => (1 << 90, 0),
		Role::TreeItem => (1 << 91, 0),
		Role::DocumentSpreadsheet => (1 << 92, 0),
		Role::DocumentPresentation => (1 << 93, 0),
		Role::DocumentText => (1 << 94, 0),
		Role::DocumentWeb => (1 << 95, 0),
		Role::DocumentEmail => (1 << 96, 0),
		Role::Comment => (1 << 97, 0),
		Role::ListBox => (1 << 98, 0),
		Role::Grouping => (1 << 99, 0),
		Role::ImageMap => (1 << 100, 0),
		Role::Notification => (1 << 101, 0),
		Role::InfoBar => (1 << 102, 0),
		Role::LevelBar => (1 << 103, 0),
		Role::TitleBar => (1 << 104, 0),
		Role::BlockQuote => (1 << 105, 0),
		Role::Audio => (1 << 106, 0),
		Role::Video => (1 << 107, 0),
		Role::Definition => (1 << 108, 0),
		Role::Article => (1 << 109, 0),
		Role::Landmark => (1 << 110, 0),
		Role::Log => (1 << 111, 0),
		Role::Marquee => (1 << 112, 0),
		Role::Math => (1 << 113, 0),
		Role::Rating => (1 << 114, 0),
		Role::Timer => (1 << 115, 0),
		Role::Static => (1 << 116, 0),
		Role::MathFraction => (1 << 117, 0),
		Role::MathRoot => (1 << 118, 0),
		Role::Subscript => (1 << 119, 0),
		Role::Superscript => (1 << 120, 0),
		Role::DescriptionList => (1 << 121, 0),
		Role::DescriptionTerm => (1 << 122, 0),
		Role::DescriptionValue => (1 << 123, 0),
		Role::Footnote => (1 << 124, 0),
		Role::ContentDeletion => (1 << 125, 0),
		Role::ContentInsertion => (1 << 126, 0),
		Role::Mark => (1 << 127, 0),
		Role::Suggestion => (0, 1 << 0),
		Role::PushButtonMenu => (0, 1 << 1),
	}
}

#[cfg(test)]
mod tests {
	use super::{Role, RoleSet};
	#[test]
	fn check_bit_or_assign_max_role() {
		let max_role = Role::PushButtonMenu.into();
		let mut rs = RoleSet::default();
		rs |= max_role;
		assert!(rs.contains(max_role), "{max_role:?} not found in set {rs:?}");
	}

	#[test]
	fn check_all_roles_no_error() {
		let all_roles = RoleSet::ALL;
		assert_eq!(all_roles.role_iter().count(), 130);
	}

	#[test]
	fn check_bits_and() {
		let all_roles = RoleSet::ALL;
		let no_roles = RoleSet::EMPTY;
		assert_eq!(all_roles & no_roles, no_roles);
	}

	#[test]
	fn check_bits_not() {
		let all_roles = RoleSet::ALL;
		let no_roles = RoleSet::EMPTY;
		assert_eq!(!all_roles, no_roles);
		assert_eq!(!no_roles, all_roles);
	}

	#[test]
	fn check_bits_and_assign() {
		let less_roles = RoleSet::EMPTY | Role::Frame | Role::Link;
		let mut some_roles = RoleSet::EMPTY
			| Role::Invalid | Role::Suggestion
			| Role::Link | Role::Frame;
		// initially, one way relationship between rolesets
		assert!(some_roles.contains(less_roles));
		assert!(!(less_roles.contains(some_roles)));
		some_roles &= less_roles;
		// after &=, two way relationship between rolesets
		assert!(some_roles.contains(less_roles));
		assert!(less_roles.contains(some_roles));
	}
}
