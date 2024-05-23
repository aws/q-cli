const Modal = ({
  isOpen,
  onClose,
  message,
}: {
  isOpen: boolean;
  onClose: () => void;
  message: string;
}) => {
  if (!isOpen) return null;

  return (
    <div className="flex fixed w-full h-full top-0 bottom-0 left-0 right-0 items-center justify-center z-[1000]">
      <div className="text-white dark:text-black bg-black dark:bg-white rounded flex flex-row items-center justify-center overflow-auto p-1">
        <button
          className="bg-transparent border-none text-xl cursor-pointer mx-1"
          onClick={onClose}
        >
          &times;
        </button>
        <p className="mx-1">{message}</p>
      </div>
    </div>
  );
};

export default Modal;
