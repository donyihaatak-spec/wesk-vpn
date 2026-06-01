/** Декоративный фон на весь viewport — заполняет пустоту при полном экране. */

export function AppBackdrop() {
  return (
    <div className="app-bg" aria-hidden>
      <div className="app-bg__grid" />
      <div className="app-bg__orb app-bg__orb--1" />
      <div className="app-bg__orb app-bg__orb--2" />
      <div className="app-bg__orb app-bg__orb--3" />
      <div className="app-bg__beam app-bg__beam--1" />
      <div className="app-bg__beam app-bg__beam--2" />
    </div>
  );
}
