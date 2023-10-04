import settings from "@/data/translate";

export default function Page() {
  console.log(settings);
  return (
    <>
      <section className="flex flex-col">
        <h1
          id={`subhead-predict`}
          className="font-bold text-2xl leading-none mt-2"
        >
          Predict
        </h1>
      </section>
      {/* {settings.map((section, i) => <UserPrefSection data={section} index={i} key={i} />)} */}
    </>
  );
}
